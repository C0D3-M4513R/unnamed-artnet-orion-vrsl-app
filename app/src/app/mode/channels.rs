use egui::{CentralPanel, Vec2, Widget, WidgetText};
use serde_derive::{Deserialize, Serialize};
use crate::app::{mode, OtherAppState, SerializableAppData, SubMenu};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(super) struct Channels{
    ///Mode Channels:
    view_by_device: bool,
    universe: ux2::u15,
}

fn common_slider(value: &mut u8, ui: &mut egui::Ui) {
    //todo: I don't like how those sliders look

    // let old_style = ui.style().clone();
    // let slider_style = ui.style_mut();
    egui::Slider::new(value, u8::MIN..=u8::MAX)
        .vertical()
        .handle_shape(egui::style::HandleShape::Rect {
            aspect_ratio: 1./2.
        })
        .ui(ui);
    // ui.set_style(old_style);
}

fn multiplier_slider(name: impl Into<WidgetText>, value: &mut u8, ui: &mut egui::Ui) {
    ui.label(name);
    common_slider(value, ui);
}
fn channel_slider(name: impl Into<WidgetText>, value: &mut Option<u8>, ui: &mut egui::Ui) {
    //todo: track actual slider value
    let mut channel_value = value.unwrap_or_default();
    let mut lock = value.is_some();
    ui.checkbox(&mut lock, name);
    ui.add_enabled_ui(
        lock,
        |ui| common_slider(&mut channel_value, ui)
    );
    if lock {
        *value = Some(channel_value);
    } else{
        *value = None;
    }
}

impl Channels {
    fn view_by_device(&mut self, serializable_app_data: &mut SerializableAppData, ui: &mut egui::Ui) {
        let universe = serializable_app_data.data.devices.create_or_get_universe(self.universe);
        if universe.is_empty() {
            ui.label("There are no devices in this universe.");
        }else {
            ui.label("This section is under Construction!");
        }
    }
    fn view_by_channel(&mut self, serializable_app_data: &mut SerializableAppData, ui: &mut egui::Ui) {
        ui.horizontal(|ui|{
            egui::ScrollArea::new([true, false])
                .show(ui, |ui|{
                    debug_assert_eq!((ux2::u9::MAX+ux2::u9::new(2)).into_inner(), 514);
                    let mut layout = egui::Layout::default();
                    layout.main_dir = egui::Direction::TopDown;
                    layout.main_align = egui::Align::Center;
                    egui_extras::StripBuilder::new(ui)
                        .sizes(egui_extras::Size::exact(75.), 514)
                        .cell_layout(layout)
                        .horizontal(|mut strip|{
                            strip.cell(|ui|multiplier_slider("Global\nMaster\nMultiplier", &mut serializable_app_data.data.global_multiplier, ui));
                            let universe = serializable_app_data.data.overrides.create_or_get_universe(self.universe);
                            let universe_override = &mut universe.multiplier;
                            let channels = &mut universe.channels;
                            strip.cell(|ui|multiplier_slider("Universe\nMaster\nOverride", universe_override, ui));
                            for (id, channel) in channels.iter_mut().enumerate() {
                                strip.cell(|ui|channel_slider(format!("Override\nChannel\n{}", id+1), channel, ui));
                            }
                        });
                    ui.allocate_at_least(Vec2::new(1.,175.), egui::Sense::click());
                });
        });
    }
}

impl SubMenu for Channels{
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame, serializable_app_data: &mut SerializableAppData, _: &mut OtherAppState, _: mode::AppMode) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui|{
               if ui.small_button(if self.view_by_device {"View By Channel"} else {"View By Device"}).clicked(){
                   self.view_by_device = !self.view_by_device;
               }

                ui.label("Universe: ");
                egui::DragValue::new(&mut self.universe)
                    .clamp_range(0u16..=ux2::u15::MAX.into())
                    .ui(ui)
            });

            if self.view_by_device {
                self.view_by_device(serializable_app_data, ui);
            }else{
                self.view_by_channel(serializable_app_data, ui);
            }
        });
    }
}
