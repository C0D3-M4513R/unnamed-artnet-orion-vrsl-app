use egui::{CentralPanel, Widget};
use serde_derive::{Deserialize, Serialize};
use crate::app::{mode, OtherAppState, SerializableAppData, SubMenu};
use crate::artnet::universe::UniverseError;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(super) struct Channels{
    ///Mode Channels:
    view_by_device: bool,
    view_universe: ux2::u15,
    view_universe_error: Option<UniverseError>,
}

impl SubMenu for Channels{
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame, serializable_app_data: &mut SerializableAppData, _: &mut OtherAppState, _: mode::AppMode) {
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

            let universe = serializable_app_data.data.devices.create_or_get_universe(self.view_universe);
            ui.label("This section is under Construction!");
        });
    }
}