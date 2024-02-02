use std::fmt::{Display, Formatter};
use serde_derive::{Deserialize, Serialize};
use crate::app::{mode, OtherAppState, SerializableAppData, SubMenu};

mod fixtures;
mod todo;
mod channels;

#[derive(Default, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub(super) enum AppMode{
    FixtureBuilder,
    #[default]
    Fixtures,
    Channels,
    Functions,
}
impl Display for AppMode{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FixtureBuilder => write!(f, "FixtureBuilder"),
            Self::Fixtures => write!(f, "Fixtures"),
            Self::Channels => write!(f, "Channels"),
            Self::Functions => write!(f, "Functions"),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct SubScreens {
    fixtures: fixtures::Fixtures,
    channels: channels::Channels,
}

impl SubScreens {
    pub(super) fn menu_subscreen_select(ui: &mut egui::Ui, mode: &mut AppMode){
        egui::menu::menu_button(ui, "Modes", |ui|{
            for e in [AppMode::FixtureBuilder, AppMode::Fixtures, AppMode::Channels, AppMode::Functions] {
                ui.selectable_value(mode, e, e.to_string());
            }
        });
    }
}

impl SubMenu for SubScreens {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, serializable_app_data: &mut SerializableAppData, other_app_state: &mut OtherAppState, mode: mode::AppMode) {
        match mode {
            AppMode::FixtureBuilder |
            AppMode::Functions
                => todo::Todo::default().update(ctx, frame, serializable_app_data, other_app_state, mode),
            AppMode::Fixtures => self.fixtures.update(ctx, frame, serializable_app_data, other_app_state, mode),
            AppMode::Channels => self.channels.update(ctx, frame, serializable_app_data, other_app_state, mode),
        }
    }
}