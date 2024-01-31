use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use egui::TopBottomPanel;
use egui::mutex::RwLock;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use common_data::CommonData;

mod common_data;
mod message;
mod mode;
mod storage;

const LAST_OPENED_FILE: &'static str = "LAST_OPENED_FILE";

#[derive(Default, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
enum AppMode{
    FixtureBuilder,
    #[default]
    Fixtures,
    Channels,
    Functions,
}
impl Display for AppMode{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::FixtureBuilder => write!(f, "FixtureBuilder"),
            AppMode::Fixtures => write!(f, "Fixtures"),
            AppMode::Channels => write!(f, "Channels"),
            AppMode::Functions => write!(f, "Functions"),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct App<'a>{
    #[cfg(feature = "egui_tracing")]
    logs_visible: bool,
    #[cfg(feature = "egui_tracing")]
    #[serde(skip)]
    collector:egui_tracing::EventCollector,
    #[serde(skip)]
    project_file: Option<PathBuf>,
    mode: AppMode,
    /// Data that is shared with also shared with the artnet processing thread
    #[serde(skip)]
    common_data_mutex: Arc<RwLock<CommonData>>,
    /// Clone of the last data, that has been written into the `common_data_mutex`
    common_data_copy: CommonData,
    /// Data that is edited in the gui thread, but has not been sent to the artnet thread
    data: CommonData,
    #[serde(skip)]
    channel: Option<UnboundedSender<message::Message>>,
    #[serde(skip)]
    popups: VecDeque<Box<PopupFunc<'a>>>,
}
impl<'a> Debug for App<'a>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("App");
         debug
             .field("project_file", &self.project_file)
             .field("mode", &self.mode)
             .field("common_data", &"Arc<Mutex<CommmonData>>")
             .field("channel", &self.channel)
             .field("popups.len()", &self.popups.len())
             .finish()
    }
}
impl<'a> Default for App<'a>{
    fn default() -> Self {
        Self{
            #[cfg(feature = "egui_tracing")]
            logs_visible: false,
            #[cfg(feature = "egui_tracing")]
            collector:egui_tracing::EventCollector::new(),
            project_file: None,
            mode: AppMode::default(),
            common_data_mutex: Arc::new(RwLock::new(CommonData::default())),
            common_data_copy: CommonData::default(),
            data: CommonData::default(),
            channel: None,
            popups: VecDeque::new(),
        }
    }
}

impl<'a> App<'a> {
    #[cfg(feature = "egui_tracing")]
    /// Called once before the first frame.
    pub fn new_collector(collector: egui_tracing::EventCollector, cc: &eframe::CreationContext<'_>) -> Self {
        let mut slf: App = Self::new(cc);

        #[cfg(not(debug_assertions))]
        slf.collector = collector;
        slf
    }
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        let last_opened_file_opt =
            cc.storage.map(|storage| eframe::get_value(storage, LAST_OPENED_FILE))
                .flatten();
        let mut slf: App = App::default();
        slf.project_file = last_opened_file_opt;
        //todo: load project state
        //todo: start artnet thread

        slf
    }

    fn handle_display_popup<D: std::fmt::Display>(
        &mut self,
        label: impl Into<egui::WidgetText> + 'a,
        error: &D,
        title: impl Into<egui::WidgetText> + 'a,
    ) {
        let error_string = error.to_string();
        let label = label.into().clone();
        self.popups.push_front(popup_creator(title, move |_, ui| {
            ui.label(label.clone());
            ui.label("Some developer information below:");
            ui.label(&error_string);
        }));
    }
}

impl<'a> eframe::App for App<'a> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu_bar:menu").show(ctx, |ui|{
           egui::menu::bar(ui, |ui|{
               egui::menu::menu_button(ui, "File", |ui|{

               });
               egui::menu::menu_button(ui, "Modes", |ui|{
                   for e in [AppMode::FixtureBuilder, AppMode::Fixtures, AppMode::Channels, AppMode::Functions] {
                        ui.selectable_value(&mut self.mode, e, e.to_string());
                   }
               });
           });
        });
        match self.mode {
            AppMode::FixtureBuilder => self.todo(ctx, frame),
            AppMode::Fixtures => self.fixtures(ctx, frame),
            AppMode::Channels => self.todo(ctx, frame),
            AppMode::Functions => self.todo(ctx, frame),
        }
        self.popups = core::mem::take(&mut self.popups).into_iter().filter_map(|mut popup|{
            if popup(self, ctx, frame) {
                Some(popup)
            }else{
                None
            }
        }).collect();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, LAST_OPENED_FILE, &self.project_file)
    }
}
type PopupFunc<'a> = dyn FnMut(&'_ mut App,&'_ egui::Context, &'_ mut eframe::Frame) -> bool + 'a;

fn get_id() -> u64 {
    static ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn popup_creator<'a>(
    title: impl Into<egui::WidgetText> + 'a,
    add_content: impl FnMut(&mut App, &mut egui::Ui) + 'a,
) -> Box<PopupFunc<'a>> {
    popup_creator_collapsible(title, false, add_content)
}

fn popup_creator_collapsible<'a>(
    title: impl Into<egui::WidgetText> + 'a,
    collapsible: bool,
    mut add_content: impl FnMut(&mut App, &mut egui::Ui) + 'a,
) -> Box<PopupFunc<'a>> {
    let title = title.into();
    let id = get_id();
    let mut open = true;
    Box::new(move |app:&'_ mut App,ctx: &'_ egui::Context, _: &'_ mut eframe::Frame| {
        egui::Window::new(title.clone())
            .resizable(false)
            .collapsible(collapsible)
            .open(&mut open)
            .id(egui::Id::new(id))
            .show(ctx, |ui|add_content(app,ui));
        open
    })
}
