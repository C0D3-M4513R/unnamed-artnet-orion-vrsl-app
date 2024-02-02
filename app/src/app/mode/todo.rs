use egui::CentralPanel;
use crate::app::{mode, OtherAppState, SerializableAppData, SubMenu};

#[derive(Default)]
pub(super) struct Todo;

impl SubMenu for Todo{
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame, _: &mut SerializableAppData, _: &mut OtherAppState, _: mode::AppMode) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("This section still needs to be done");
        });
    }
}