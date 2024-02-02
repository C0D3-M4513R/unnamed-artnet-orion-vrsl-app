use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use eframe::Storage;
use egui::TopBottomPanel;
use egui::mutex::RwLock;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use common_data::CommonData;
use crate::app::mode::SubScreens;
use crate::app::popup::{handle_display_popup, popup_creator_raw};
use crate::app::storage::FileStore;
use crate::fixturestore::FixtureStore;
use crate::get_runtime;

mod common_data;
mod message;
mod mode;
mod storage;
mod popup;

const LAST_OPENED_FILE: &str = "LAST_OPENED_FILE";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct App{
    mode: mode::AppMode,
    sub_screens: mode::SubScreens,
    /// Data that is shared with also shared with the artnet processing thread
    serializable_app_data: SerializableAppData,
    #[serde(skip)]
    other_app_state: OtherAppState,
}

#[derive(Default)]
pub struct OtherAppState{
    pub(self) file_store: Arc<RwLock<FileStore>>,
    pub(self) project_file: Option<Arc<Path>>,
    ///invariant: common_data_mutex==common_data_copy
    /// The artnet thread should thus take care not to modify that data.
    pub(self) common_data_mutex: Arc<RwLock<CommonData>>,
    //todo: do we really need this?
    pub(self) channel: Option<UnboundedSender<message::Message>>,
    pub(self) popups: popup::ArcPopupStore,
    _marker: PhantomData<()>, //not_exhaustive
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SerializableAppData{
    pub(self) fixture_store: Arc<FixtureStore>,
    /// Clone of the last data, that has been written into the `common_data_mutex`
    pub(self) common_data_copy: CommonData,
    /// Data that is edited in the gui thread, but has not been sent to the artnet thread
    pub(self) data: CommonData,
    _marker: PhantomData<()>, //not_exhaustive

}

impl Debug for OtherAppState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("App");
        debug
            .field("file_store", &"...")
            .field("project_file", &self.project_file)
            .field("common_data_mutex", &"...")
            .field("channel", &"...")
            .field("popups.len()", &"...")
            .finish()
    }
}

async fn get_file_store(path: Arc<Path>, popups: Option<&mut popup::PopupStore>) -> Result<FileStore, FileStore> {
    let (err, fs) = FileStore::from_ron_filepath(path.clone()).await;
    if let Some(err) = err {
        log::warn!("Failed to get File store of path \"{:?}\", because: {}", &path, &err);
        if let Some(popups) = popups {
            handle_display_popup(
                popups,
                "There was an error loading the Project Data.",
                &err,
                "Error loading Project Data"
            )
        }
        Err(fs)
    }else {
        Ok(fs)
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        let mut popups = VecDeque::default();
        let last_opened_file_opt = match match cc.storage {
            None => {
                popups.push_front(popup_creator_raw("No Persistance", move |_, ui| {
                    ui.label("Unable to determine last opened project, due to no system app storage being given.");
                    ui.label("You may still open a project manually.");
                }));
                None
            }
            Some(storage) => Some(eframe::get_value::<PathBuf>(storage, LAST_OPENED_FILE))
        }{
            None => None,
            Some(None) => {
                popups.push_front(popup_creator_raw("No last Opened Project?", move |_, ui| {
                    ui.label("Unable to get the path of the last opened Project.");
                    ui.label("You may still open a project manually.");
                }));
                None
            },
            Some(Some(last_project)) => Some(Arc::from(last_project))
        };
        let file_store = last_opened_file_opt.as_ref().map_or_else(
            FileStore::default,
            |last_opened_file: &Arc<Path>| get_runtime().block_on(
                get_file_store(last_opened_file.clone(), Some(&mut popups))
            ).unwrap_or_else(|v| v)
        );
        Self::with_file_store(file_store, last_opened_file_opt, popups)
    }

    pub fn with_file_store(file_store: FileStore, last_opened_file_opt: Option<Arc<Path>>, mut popups: popup::PopupStore) -> Self {
        let mut app:Option<Self> = None;

        match file_store.get_string("app") {
            None => {
                if last_opened_file_opt.is_some(){
                    handle_display_popup(
                        &mut popups,
                        "Error Opening Last Project",
                        &"Unable to read app data from storage",
                        "Unable to read/open last Project"
                    )
                }
            }
            Some(app_data) => {
                match ron::de::from_str(app_data.as_str()){
                    Ok(app_de) => app = Some(app_de),
                    Err(err) => {
                        handle_display_popup(
                            &mut popups,
                            "Error Opening Last Project",
                            &err,
                            "Unable to parse app data from Project File",
                        );
                    }
                }
            }
        }
        let mut slf = app.unwrap_or_default();
        slf.serializable_app_data.fixture_store.populate_fixture_store_defaults();
        slf.other_app_state.file_store = Arc::new(RwLock::new(file_store));
        slf.other_app_state.project_file = last_opened_file_opt;
        slf.other_app_state.common_data_mutex = Arc::new(RwLock::new(slf.serializable_app_data.common_data_copy.clone()));
        slf.other_app_state.popups = Arc::new(Mutex::new(popups));
        //todo: start artnet thread

        slf
    }

    fn sync_changes(&mut self) {
        //todo: is this fine? This depends on the artnet thread.
        // It should take care to hold the read guard for as little time as possible.
        let mut write_guard = self.other_app_state.common_data_mutex.write();
        *write_guard = self.serializable_app_data.data.clone();
        self.serializable_app_data.common_data_copy = self.serializable_app_data.data.clone();
        drop(write_guard);
    }

    fn display_popups(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame){
        let mut old_popup = core::mem::take(
            //Speed: there should never be a long lock on the popups. (only to push basically)
            &mut *get_runtime().block_on(
                self.other_app_state.popups.lock()
            )
        );
        //we intentionally release the lock here. otherwise self would partly be borrowed, which will disallow the popup closure call
        let mut new_popup = old_popup.into_iter().filter_map(|mut popup|{
            if popup(self, ctx, frame) {
                Some(popup)
            }else{
                None
            }
        }).collect();
        //Speed: there should never be a long lock on the popups. (only to push basically)
        let mut lock = get_runtime().block_on(self.other_app_state.popups.lock());
        core::mem::swap(&mut *lock, &mut new_popup);
        lock.append(&mut new_popup);
        drop(lock);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu_bar:menu").show(ctx, |ui|{
           egui::menu::bar(ui, |ui|{
               egui::menu::menu_button(ui, "File", |ui|{
                   if ui.button("Save").clicked(){
                        self.other_app_state.file_store.write().flush();
                   }
                   if ui.button("Open").clicked(){
                        todo!("add open functionality")
                   }
               });
               SubScreens::menu_subscreen_select(ui, &mut self.mode);
               if ui.button("Apply Pending Changes").clicked() {
                   self.sync_changes();
               }
           });
        });
        self.sub_screens.update(ctx, frame, &mut self.serializable_app_data, &mut self.other_app_state, self.mode);

    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, LAST_OPENED_FILE, &self.other_app_state.project_file)
    }
}
trait SubMenu {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, serializable_app_data: &mut SerializableAppData, other_app_state: &mut OtherAppState, mode: mode::AppMode);
}