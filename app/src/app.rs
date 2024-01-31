use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct App<'a>{
    #[cfg(feature = "egui_tracing")]
    logs_visible: bool,
    #[cfg(feature = "egui_tracing")]
    #[serde(skip)]
    collector:egui_tracing::EventCollector,
    #[serde(skip)]
    popups: VecDeque<Box<PopupFunc<'a>>>,
}
impl<'a> Debug for App<'a>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("App");
         debug
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

        let slf: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        #[cfg(not(debug_assertions))]
        log::info!("You are running a release build. Some log statements were disabled.");
        slf
    }

    fn handle_join_error(
        &mut self,
        error: &tokio::task::JoinError,
        title: impl Into<egui::WidgetText> + 'a,
    ) {
        self.handle_display_popup("An unknown error occurred while logging out.", error, title);
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
        self.popups = core::mem::take(&mut self.popups).into_iter().filter_map(|mut popup|{
            if popup(self, ctx, frame) {
                Some(popup)
            }else{
                None
            }
        }).collect();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage,eframe::APP_KEY, self)
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
