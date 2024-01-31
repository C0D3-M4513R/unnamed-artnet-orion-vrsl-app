use egui::CentralPanel;
use crate::app::App;

impl<'a> App<'a>{
    pub(in super::super) fn todo(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("This section still needs to be done");
        });
    }
}