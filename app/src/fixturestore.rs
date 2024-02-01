use std::sync::Arc;
use dashmap::{DashMap, DashSet};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use crate::artnet::fixture::channel::{Channel, Action, Color, ColorRGB, Range, SimpleAction};
use crate::artnet::fixture::Fixture;
use crate::artnet::fixture::variables::{Variable, VariableChannelAction};
use crate::degree::deg_to_microarcseconds;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct FixtureStore{
    fixtures: DashSet<Fixture>,
    contained_paths: DashMap<Arc<str>, FixtureStore>,
}

impl FixtureStore{

    pub(crate) fn populate_fixture_store_defaults(&self){
        for fixture in [&VRSL_PAR_LIGHT, &VRSL_BAR_LIGHT, &VRSL_BLINDER, &VRSL_MOVING_HEAD, &VRSL_LASER]{
            self.put_path(fixture.get_path().as_ref(), (*fixture).clone());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty() && self.contained_paths.is_empty()
    }
    fn get_path<R>(&self, path: &[Arc<str>], func: impl FnOnce(&Self)->R) -> R {
        //I cannot solve this without taking a callback and doing it recursively.
        //When trying to not do this recursively or to not take a callback,
        // rust complains about temporary reference lifetimes.
        match path.split_first() {
            None => func(self),
            Some((first, tail)) => {
                self.contained_paths
                    .entry(first.clone())
                    .or_default()
                    .value()
                    .get_path(tail, func)
            },
        }
    }

    fn put_path(&self, path: &[Arc<str>], fixture: Fixture) {
        self.get_path(path, |fs|fs.fixtures.insert(fixture));
    }

    #[allow(clippy::significant_drop_tightening, clippy::significant_drop_in_scrutinee)]//false positive for items
    fn add_contained_fixtures(&self, ui: &mut egui::Ui,path: &mut Vec<Arc<str>>, item: &mut (Vec<Arc<str>>, Option<Fixture>)) {
        let mut items = self.fixtures.iter().collect::<Vec<_>>();
        items.sort_by_cached_key(|x|x.key().get_model().clone());
        for i in items {
            let key = i.key();
            path.push(key.get_model().clone());
            if ui.selectable_label(
                item.1.as_ref().is_some_and(|f|f==key),
                key.get_model().as_ref()
            ).clicked() {
                *item = (path.clone(), Some(key.clone()));
            }
            path.pop();
        }
    }
    #[allow(clippy::significant_drop_tightening, clippy::significant_drop_in_scrutinee)]//false positive for items
    pub fn _build_menu(&self, ui: &mut egui::Ui, path: &mut Vec<Arc<str>>, item: &mut (Vec<Arc<str>>, Option<Fixture>)){
        self.add_contained_fixtures(ui,  path, item);
        let mut items = self.contained_paths.iter().collect::<Vec<_>>();
        items.sort_by_cached_key(|x|x.key().clone());
        for element in items{
            let value = element.value();
            if value.is_empty() {continue}
            let key = element.key();

            path.push(key.clone());
            ui.menu_button(key.as_ref(), |ui|{
                value._build_menu(ui, path, item);
            });
            path.pop();
        }
    }

    #[inline]
    pub fn build_menu(&self, ui: &mut egui::Ui, item: &mut (Vec<Arc<str>>, Option<Fixture>)){
        self._build_menu(ui, &mut Vec::new(), item)
    }
}


pub static FIXTURE_STORE:Lazy<FixtureStore> = Lazy::new(||FixtureStore {
    fixtures: DashSet::new(),
    contained_paths: DashMap::new(),
});

//<editor-fold desc="Fixture definitions" defaultstate="collapsed">
macro_rules! create_arc {
    ($( $ident:ident, $str:literal ),+) => {
        $( static $ident:Lazy<Arc<str>> = Lazy::new(||Arc::from($str)); )+
    };
}
create_arc!(
    VRSL, "VRSL",
    STANDARD_MOVER_SPOTLIGHT, "Standard Mover Spotlight",
    MOVING_HEAD, "Moving Head",
    STANDARD_LASER, "Standard Laser",
    LASER, "Laser"
);
static VRSL_PANS:Lazy<Arc<[u64]>> = Lazy::new(||Arc::from([
    deg_to_microarcseconds(180),
    deg_to_microarcseconds(360),
    deg_to_microarcseconds(540)
]));
static VRSL_TILTS:Lazy<Arc<[u64]>> = Lazy::new(||Arc::from([
    deg_to_microarcseconds(180),
    deg_to_microarcseconds(250),
    deg_to_microarcseconds(270)
]));
static VRSL_MOVING_HEAD:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    STANDARD_MOVER_SPOTLIGHT.clone(),
    MOVING_HEAD.clone(),
    Arc::new([
        Channel::new_simple(SimpleAction::VariableChannelAction(VariableChannelAction::PositionPan(Variable::Selection(VRSL_PANS.clone(), 180)))),
        //todo: vrsl - currently disabled, because the linear smoothing algorithm outweighs this
        Channel::new_simple(SimpleAction::VariableChannelAction(VariableChannelAction::PositionPanFine(Variable::Set(0)))),
        Channel::new_simple(SimpleAction::VariableChannelAction(VariableChannelAction::PositionTilt(Variable::Selection(VRSL_TILTS.clone(), 180)))),
        //todo: vrsl - currently disabled, because the linear smoothing algorithm outweighs this
        Channel::new_simple(SimpleAction::VariableChannelAction(VariableChannelAction::PositionTiltFine(Variable::Set(0)))),
        Channel::new_simple(SimpleAction::BeamZoom),
        Channel::new_simple(SimpleAction::IntensityMasterDimmer),
        Channel::new(Action::Selection(Arc::new([
            Range::new(0, 9, SimpleAction::NoOp),
            Range::new(10, 255, SimpleAction::Strobo),
        ]))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Red))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Green))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Blue))),
        Channel::new(Action::Selection(Arc::new([
            Range::new(0, 9, SimpleAction::NoOp),
            Range::new(10, 126, SimpleAction::SpinLeft),
            Range::new(127, 255, SimpleAction::SpinRight),
        ]))),
        Channel::new(Action::Selection(Arc::new([
            Range::new(0, 42, SimpleAction::GOBOSelection),
            Range::new(43, 85, SimpleAction::GOBOSelection),
            Range::new(86, 127, SimpleAction::GOBOSelection),
            Range::new(128, 212, SimpleAction::GOBOSelection),
            Range::new(213, 255, SimpleAction::GOBOSelection),
        ]))),
        Channel::new_simple(SimpleAction::Speed),
    ]),
));
static VRSL_PAR_LIGHT:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    Arc::from("Standard Par Light"),
    Arc::from("Color Changer"),
    Arc::new([
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::IntensityMasterDimmer),
        Channel::new(Action::Selection(Arc::new([
            Range::new(0, 9, SimpleAction::NoOp),
            Range::new(10, 255, SimpleAction::Strobo),
        ]))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Red))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Green))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Blue))),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
    ]),
));
static VRSL_BAR_LIGHT:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    Arc::from("Standard BarLight"),
    Arc::from("LED Bar (Pixels)"),
    Arc::new([
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::IntensityMasterDimmer),
        Channel::new(Action::Selection(Arc::new([
            Range::new(0, 9, SimpleAction::NoOp),
            Range::new(10, 255, SimpleAction::Strobo),
        ]))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Red))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Green))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Blue))),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
    ]),
));
static VRSL_BLINDER:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    Arc::from("Standard Blinder"),
    Arc::from("Strobe"),
    Arc::new([
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::IntensityMasterDimmer),
        Channel::new(Action::Selection(Arc::new([
            Range::new(0, 9, SimpleAction::NoOp),
            Range::new(10, 255, SimpleAction::Strobo),
        ]))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Red))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Green))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Blue))),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
        Channel::new_simple(SimpleAction::NoOp),
    ]),
));
static VRSL_LASER:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    STANDARD_LASER.clone(),
    LASER.clone(),
    Arc::new([
        Channel::new_simple(SimpleAction::VariableChannelAction(VariableChannelAction::PositionPan(Variable::Selection(VRSL_PANS.clone(), 180)))),
        Channel::new_simple(SimpleAction::VariableChannelAction(VariableChannelAction::PositionTilt(Variable::Selection(VRSL_TILTS.clone(), 180)))),
        Channel::new_simple(SimpleAction::NoOp), //todo: laser width
        Channel::new_simple(SimpleAction::NoOp), //todo: laser flatness
        Channel::new_simple(SimpleAction::NoOp), //todo: beam count
        Channel::new_simple(SimpleAction::SpinLeft), //todo: spin left or right?
        Channel::new_simple(SimpleAction::IntensityMasterDimmer),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Red))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Green))),
        Channel::new_simple(SimpleAction::IntensityColor(Color::Rgb(ColorRGB::Blue))),
        Channel::new_simple(SimpleAction::NoOp), //todo: beam thickness
        Channel::new_simple(SimpleAction::NoOp), //todo: length
        Channel::new_simple(SimpleAction::Speed),
    ]),
));
//</editor-fold>