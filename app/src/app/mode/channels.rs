use egui::{CentralPanel, Widget};
use crate::app::App;

impl<'a> App<'a> {
    pub(in super::super) fn channels(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui|{
               if ui.small_button(if self.view_by_device {"View By Channel"} else {"View By Device"}).clicked(){
                   self.view_by_device = !self.view_by_device;
               }

                ui.label("Universe: ");
                let ui_universe = {
                    #[allow(clippy::range_minus_one)] //bad suggestion - would lead to compilation error
                        {
                        egui::DragValue::new(&mut self.view_universe)
                            .clamp_range(0u16..=ux2::u15::MAX.into())
                            .ui(ui)
                    }
                };

                if let Some(err) = self.view_universe_error {
                    ui_universe.ctx.debug_painter().error(ui_universe.rect.left_bottom(), err);
                }
            });

            let universe = self.data.create_or_get_universe(self.view_universe);
            ui.label("This section is under Construction!");
        });
    }
}