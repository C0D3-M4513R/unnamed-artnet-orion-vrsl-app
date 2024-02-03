use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use eframe::Storage;
use egui::TopBottomPanel;
use egui::mutex::RwLock;
use rfd::FileHandle;
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
    #[cfg(feature = "puffin")]
    #[serde(skip)]
    debug: bool,
    #[serde(skip)]
    other_app_state: OtherAppState,
}

#[derive(Debug)]
enum FileDialog{
    ProjectSaveNew,
    ProjectOpen,
}
impl Display for FileDialog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProjectSaveNew => write!(f, "Save New"),
            Self::ProjectOpen => write!(f, "Open"),
        }
    }
}
#[derive(Default)]
pub struct OtherAppState{
    pub(self) file_store: Arc<RwLock<FileStore>>,
    pub(self) project_file: Option<Arc<Path>>,
    pub(self) project_file_dialog: Option<(FileDialog, tokio::task::JoinHandle<Option<FileHandle>>)>,
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
            .field("project_file_dialog", &self.project_file_dialog)
            .field("common_data_mutex", &"...")
            .field("channel", &"...")
            .field("popups", &"...")
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
    /// checks if `self.other_app_state.project_file_save_dialog` is set.
    /// If it is, it will check, if the future is finished and deal with the result and it's action
    fn check_app_save_new(&mut self) {
        if let Some((action, thread)) = self.other_app_state.project_file_dialog.take() {
            if thread.is_finished(){
                //this is fine, because the thread already finished. So this should be a relatively short wait.
                match get_runtime().block_on(thread) {
                    Ok(Some(path)) => {
                        let path:Arc<Path> = Arc::from(path.path());
                        match action{
                            FileDialog::ProjectSaveNew => {
                                let mut lock = self.other_app_state.file_store.write();
                                //todo: this is not ideal.
                                #[allow(clippy::single_match_else)] //idk. I prefer a match over an if here.
                                match get_runtime().block_on(lock.change_ron_file(path.clone())){
                                    Err(err) => {
                                        log::warn!("Could not save to new location: {}. Not updating location.", &err);
                                        popup::handle_display_popup_arc(
                                            &self.other_app_state.popups,
                                            "Couldn't Save the current Project at the selected Location.",
                                            &err,
                                            "Couldn't Save to File"
                                        )
                                    },
                                    Ok(()) =>{
                                        log::info!("Changed project save location to: {:?}", path.as_ref());
                                        self.other_app_state.project_file = Some(path);
                                        popup::popup_creator(
                                            self.other_app_state.popups.clone(),
                                            "Updated Project Location",
                                            |_, ui|{
                                                ui.label("The Project Location has been successfully updated.");
                                            }
                                        )
                                    }
                                }
                            }
                            FileDialog::ProjectOpen => {
                                //todo: this is not ideal
                                let (err, fs) = get_runtime().block_on(FileStore::from_ron_filepath(path.clone()));
                                #[allow(clippy::single_match_else)] //idk. I prefer a match over an if here.
                                match err {
                                    Some(err) => {
                                        log::warn!("Could not read project at new location {:?}: {}. Refusing to load new project.", &path, &err);
                                        popup::handle_display_popup_arc(
                                            &self.other_app_state.popups,
                                            "Couldn't Save the current Project at the selected Location.",
                                            &err,
                                            "Couldn't Save to File"
                                        )
                                    }
                                    None => {
                                        self.save_impl();
                                        self.other_app_state.file_store.write().flush(Some(self.other_app_state.popups.clone()));
                                        //todo: does this work?
                                        *self = Self::with_file_store(fs, Some(path), VecDeque::new());
                                    }
                                }
                            }
                        }
                    },
                    Ok(None) => {
                        log::info!("No File Selected in File Section Dialog");
                        popup::popup_creator(
                            self.other_app_state.popups.clone(),
                            "No File Selected",
                            |_, ui|{
                                ui.label("No File Selected. Nothing will be changed.");
                            }
                        )
                    },
                    Err(err) => {
                        log::error!("An unexpected, critical error occurred during the Save Dialog {}", &err);
                        popup::handle_display_popup_arc(
                            &self.other_app_state.popups,
                            "An unexpected, critical error occurred during the Save Dialog.",
                            &err,
                            "Error during Save Dialog"
                        )
                    }
                }
            }else {
                self.other_app_state.project_file_dialog = Some((action, thread));
            }
        }
    }
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        let mut popups = VecDeque::default();
        let last_opened_file_opt = match cc.storage.map_or_else(|| {
            popups.push_front(popup_creator_raw("No Persistance", move |_, ui| {
                ui.label("Unable to determine last opened project, due to no system app storage being given.");
                ui.label("You may still open a project manually.");
            }));
            None
        }, |storage|
            Some(eframe::get_value::<PathBuf>(storage, LAST_OPENED_FILE))
        ){
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
        let old_popup = core::mem::take(
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

    fn save_impl(&self) {
        match ron::ser::to_string(&self) {
            Ok(v) => {
                let mut store = self.other_app_state.file_store.write();
                store.set_string("app", v);
            }
            Err(err) => {
                log::error!("Error serializing app data. Err: {}\n App:{:?}", &err, &self);
                popup::handle_display_popup_arc(
                    &self.other_app_state.popups,
                    "There was an error converting the Project-Data to a format that can be saved.",
                    &err,
                    "Error Serializing Project Data"
                )
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.check_app_save_new();
        TopBottomPanel::top("menu_bar:menu").show(ctx, |ui|{
           egui::menu::bar(ui, |ui|{
               egui::menu::menu_button(ui, "File", |ui|{
                   let save = ui.button("Save").clicked();
                   let save_new = ui.button("Save as New Project").clicked();
                   if save||save_new {
                       let lock = self.other_app_state.file_store.read();
                       if save && lock.get_file_path().is_none() {
                           drop(lock);
                           let mut lock = self.other_app_state.file_store.write();
                           self.save_impl();
                           lock.flush(Some(self.other_app_state.popups.clone()));
                       } else {
                           let mut rfd = rfd::AsyncFileDialog::new();
                           if let Some(path) = lock.get_file_path().and_then(|path|path.parent()){
                               rfd = rfd.set_directory(path)
                           }
                           drop(lock);
                           self.other_app_state.project_file_dialog = Some((FileDialog::ProjectSaveNew,
                               tokio::spawn(
                                   rfd.set_file_name("project.ron")
                                       .save_file()
                               )
                           ));
                       }
                   }

                   if ui.button("Open").clicked(){
                       let mut rfd = rfd::AsyncFileDialog::new();
                       if let Some(path) = self.other_app_state.file_store.read().get_file_path().and_then(|path|path.parent()){
                           rfd = rfd.set_directory(path);
                       }
                       self.other_app_state.project_file_dialog = Some(
                           (
                               FileDialog::ProjectOpen,
                               tokio::spawn(
                                   rfd.add_filter("Ron Files", &["ron"])
                                       .pick_file()
                               )
                           )
                       );
                   }

                   if ui.button("Reset").clicked(){
                       *self = Self::with_file_store(FileStore::default(), None, VecDeque::new())
                   }
               });
               SubScreens::menu_subscreen_select(ui, &mut self.mode);
               ui.add_enabled_ui(self.serializable_app_data.data != self.serializable_app_data.common_data_copy, |ui|{
                   if ui.button("Apply Pending Changes").clicked() {
                       self.sync_changes();
                   }
               });
           });
        });
        self.sub_screens.update(ctx, frame, &mut self.serializable_app_data, &mut self.other_app_state, self.mode);
        self.display_popups(ctx, frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, LAST_OPENED_FILE, &self.other_app_state.project_file);
        self.save_impl();
    }
}
trait SubMenu {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, serializable_app_data: &mut SerializableAppData, other_app_state: &mut OtherAppState, mode: mode::AppMode);
}