use eframe::Frame;
use egui::Context;
use serde_derive::{Deserialize, Serialize};
use crate::app::{OtherAppState, SerializableAppData, SubMenu};
use crate::app::mode::AppMode;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(crate) struct Settings{}

impl SubMenu for Settings{
    fn update(&mut self, ctx: &Context, frame: &mut Frame, serializable_app_data: &mut SerializableAppData, other_app_state: &mut OtherAppState, mode: AppMode) {

    }
}