use std::ops::Add;
use std::sync::Arc;
use egui::{CentralPanel, Widget};
use serde_derive::{Deserialize, Serialize};
use crate::app::{mode, OtherAppState, SerializableAppData, SubMenu};
use crate::app::popup::{get_id, popup_creator};
use crate::artnet::fixture::{Device, Fixture};
use crate::fixturestore::FIXTURE_STORE;

#[derive(Debug, Default, Copy, Clone, Deserialize, Serialize)]
pub(super) struct Fixtures;

impl Fixtures{
    fn open_add_fixture_ui(other_app_state: &mut OtherAppState) {
        let mut name = "";
        let mut universe = 1;
        let mut start_id = 0;
        let mut device_err = None;
        let mut device_insert_err = None;
        let mut opt_fixture = (Vec::<Arc<str>>::new(), None);
        let grid_id = get_id();
        popup_creator(other_app_state.popups.clone(), "Add Fixture", move |app, ui|{
            egui::Grid::new(grid_id)
                .show(ui, |ui|{
                    ui.label("Name: ");
                    ui.text_edit_singleline(&mut name);
                    ui.end_row();

                    ui.label("Universe: ");

                    egui::DragValue::new(&mut universe)
                        .clamp_range(0u16..=ux2::u15::MAX.into())
                        .ui(ui);
                    ui.end_row();

                    ui.label("Start Channel: ");
                    let max_channel =
                        ux2::u9::checked_sub(
                            ux2::u9::MAX,
                            opt_fixture.1.as_ref().map_or(
                                ux2::u9::MIN,
                                |x:&Fixture| ux2::u9::try_from(
                                    x.get_channels().len()
                                ).unwrap_or(ux2::u9::MAX)
                            )).unwrap_or(ux2::u9::MIN);
                    let ui_channel = {
                        egui::DragValue::new(&mut start_id)
                            .clamp_range(0u16..=max_channel.into())
                            .ui(ui)
                    };
                    ui.end_row();

                    ui.menu_button(format!("{} Fixture", if opt_fixture.0.is_empty() {"Set"} else {"Change"}), |ui|{
                        FIXTURE_STORE.build_menu(ui, &mut opt_fixture);
                    });
                    let vec_unset = opt_fixture.0.is_empty();
                    if vec_unset {
                        ui.label("unset");
                    }else{
                        let mut path = String::new();
                        for i in &opt_fixture.0{
                            path.push('/');
                            path.push_str(i.as_ref());
                        }
                        ui.label(path);
                    }
                    ui.end_row();
                    ui.horizontal(|ui|{
                        if let Some(fixture) = &opt_fixture.1 {
                            if ui.button("Add").clicked() {
                                match Device::new_u16(Arc::from(name), start_id, fixture.clone()) {
                                    Ok(device) => {
                                        device_err = None;
                                        let universe = app.serializable_app_data.data.devices.create_or_get_universe(ux2::u15::new(universe));
                                        match universe.try_insert(device){
                                            Ok(()) => {
                                                device_insert_err = None;
                                            },
                                            Err(err) => {
                                                device_insert_err = Some(err);
                                            }
                                        }
                                    },
                                    Err(err) => device_err = Some(err),
                                };
                            }
                        }

                        if let Some(err) = &device_err{
                            ui_channel.ctx.debug_painter().error(ui_channel.rect.left_bottom(), err);
                        }
                        if let Some(err) = device_insert_err{
                            ui_channel.ctx.debug_painter().error(ui_channel.rect.left_bottom(), err);
                        }

                        if ui.button("Clear").clicked() {
                            universe = 1;
                            start_id = 0;
                            opt_fixture = (Vec::new(), None);
                        }
                    });
                });
        });
    }
}

impl SubMenu for Fixtures{
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame, serializable_app_data: &mut SerializableAppData, other_app_state: &mut OtherAppState, _: mode::AppMode) {
        let mut remove_list = Vec::new();
        CentralPanel::default().show(ctx, |ui| {
            for (universe, devices) in serializable_app_data.data.devices.iter().enumerate() {
                if devices.is_empty() {continue;}
                let universe_str =format!("Universe {universe}");
                ui.collapsing(&universe_str, |ui|{
                    egui::Grid::new("fixtures:".to_string().add(universe_str.as_str()))
                        .num_columns(5)
                        .show(ui, |ui|{
                            ui.label("Device Id");
                            ui.label("Fixture Name");
                            ui.label("Start Channel");
                            ui.label("End Channel");
                            ui.label("Action");
                            ui.end_row();
                            for (dev_id, device) in devices.iter().enumerate(){
                                ui.label(dev_id.to_string());
                                ui.label(device.fixture.get_model().as_ref());
                                ui.label(device.start_channel().to_string());
                                ui.label(device.end_channel().to_string());
                                if ui.button("Remove").clicked() {
                                    remove_list.push((universe, dev_id));
                                }
                                //todo: add edit button
                                ui.end_row();
                            }
                        });
                });
            }
            if serializable_app_data.data.devices.is_empty(){
                ui.label("No Fixtures have been added in any Univserse. Please get started, by adding a Fixture.");
            }
            for (universe, device_id) in remove_list{
                if let Some(devices) = serializable_app_data.data.devices.get_mut(universe){
                    devices.remove(device_id);
                }
            }
            if ui.button("Add Fixture").clicked() {
                Fixtures::open_add_fixture_ui(other_app_state)
            }
        });
    }
}