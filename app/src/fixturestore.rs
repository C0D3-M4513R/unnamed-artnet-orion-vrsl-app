use std::sync::Arc;
use dashmap::{DashMap, DashSet};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use crate::artnet::fixture::channel::{Channel, ChannelAction, Color, ColorRGB, Range, SimpleChannelAction};
use crate::artnet::fixture::Fixture;
use crate::artnet::fixture::variables::Variable;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct FixtureStore{
    fixtures: DashSet<Fixture>,
    contained_paths: DashMap<Arc<str>, FixtureStore>,
}

impl FixtureStore{
    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty() && self.contained_paths.is_empty()
    }
    fn get_path<R>(&self, path: &[Arc<str>], func: impl FnOnce(&FixtureStore)->R) -> R {
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

    fn add_contained_fixtures(&self, ui: &mut egui::Ui,path: &mut Vec<Arc<str>>, item: &mut (Vec<Arc<str>>, Option<Fixture>)) {
        let mut items = self.fixtures.iter().collect::<Vec<_>>();
        items.sort_by_cached_key(|x|x.key().get_model().clone());
        for i in items.into_iter(){
            let key = i.key();
            path.push(key.get_model().clone());
            if ui.selectable_label(
                item.1.as_ref().map(|f|f==key).unwrap_or(false),
                key.get_model().as_ref()
            ).clicked() {
                *item = (path.clone(), Some(key.clone()));
            }
            path.pop();
        }
    }
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
        $( const $ident:Lazy<Arc<str>> = Lazy::new(||Arc::from($str)); )+
    };
}
create_arc!(
    VRSL, "VRSL",
    STANDARD_MOVER_SPOTLIGHT, "Standard Mover Spotlight",
    MOVING_HEAD, "Moving Head",
    STANDARD_LASER, "Standard Laser",
    LASER, "Laser"
);
const VRSL_PANS:Lazy<Arc<[usize]>> = Lazy::new(||Arc::from([180,360,540]));
const VRSL_TILTS:Lazy<Arc<[usize]>> = Lazy::new(||Arc::from([180,250,270]));
const VRSL_MOVING_HEAD:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    STANDARD_MOVER_SPOTLIGHT.clone(),
    MOVING_HEAD.clone(),
    Arc::new([
        Channel::new_simple(SimpleChannelAction::PositionPan(Variable::Selection(VRSL_PANS.clone()))),
        //todo: vrsl - currently disabled, because the linear smoothing algorithm outweighs this
        Channel::new_simple(SimpleChannelAction::PositionPanFine(Variable::Set(0))),
        Channel::new_simple(SimpleChannelAction::PositionTilt(Variable::Selection(VRSL_TILTS.clone()))),
        //todo: vrsl - currently disabled, because the linear smoothing algorithm outweighs this
        Channel::new_simple(SimpleChannelAction::PositionTiltFine(Variable::Set(0))),
        Channel::new_simple(SimpleChannelAction::BeamZoom),
        Channel::new_simple(SimpleChannelAction::IntensityMasterDimmer),
        Channel::new(ChannelAction::Selection(Arc::new([
            Range::new(false, 0, 9, SimpleChannelAction::NoOp),
            Range::new(true, 10, 255, SimpleChannelAction::Strobo),
        ]))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Red))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Green))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Blue))),
        Channel::new(ChannelAction::Selection(Arc::new([
            Range::new(false, 0, 9, SimpleChannelAction::NoOp),
            Range::new(true, 10, 126, SimpleChannelAction::SpinLeft),
            Range::new(true, 127, 255, SimpleChannelAction::SpinRight),
        ]))),
        Channel::new(ChannelAction::Selection(Arc::new([
            Range::new(false, 0,42, SimpleChannelAction::GOBOSelection),
            Range::new(false, 43, 85, SimpleChannelAction::GOBOSelection),
            Range::new(false, 86, 127, SimpleChannelAction::GOBOSelection),
            Range::new(false, 128, 212, SimpleChannelAction::GOBOSelection),
            Range::new(false, 213, 255, SimpleChannelAction::GOBOSelection),
        ]))),
        Channel::new_simple(SimpleChannelAction::Speed),
    ]),
));
const VRSL_PAR_LIGHT:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    Arc::from("Standard Par Light"),
    Arc::from("Color Changer"),
    Arc::new([
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::IntensityMasterDimmer),
        Channel::new(ChannelAction::Selection(Arc::new([
            Range::new(false, 0, 9, SimpleChannelAction::NoOp),
            Range::new(true, 10, 255, SimpleChannelAction::Strobo),
        ]))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Red))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Green))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Blue))),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
    ]),
));
const VRSL_BAR_LIGHT:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    Arc::from("Standard BarLight"),
    Arc::from("LED Bar (Pixels)"),
    Arc::new([
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::IntensityMasterDimmer),
        Channel::new(ChannelAction::Selection(Arc::new([
            Range::new(false, 0, 9, SimpleChannelAction::NoOp),
            Range::new(true, 10, 255, SimpleChannelAction::Strobo),
        ]))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Red))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Green))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Blue))),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
    ]),
));
const VRSL_BLINDER:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    Arc::from("Standard Blinder"),
    Arc::from("Strobe"),
    Arc::new([
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::IntensityMasterDimmer),
        Channel::new(ChannelAction::Selection(Arc::new([
            Range::new(false, 0, 9, SimpleChannelAction::NoOp),
            Range::new(true, 10, 255, SimpleChannelAction::Strobo),
        ]))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Red))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Green))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Blue))),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
        Channel::new_simple(SimpleChannelAction::NoOp),
    ]),
));
const VRSL_LASER:Lazy<Fixture> = Lazy::new(||Fixture::new(
    VRSL.clone(),
    STANDARD_LASER.clone(),
    LASER.clone(),
    Arc::new([
        Channel::new_simple(SimpleChannelAction::PositionPan(Variable::Selection(VRSL_PANS.clone()))),
        Channel::new_simple(SimpleChannelAction::PositionTilt(Variable::Selection(VRSL_TILTS.clone()))),
        Channel::new_simple(SimpleChannelAction::NoOp), //todo: laser width
        Channel::new_simple(SimpleChannelAction::NoOp), //todo: laser flatness
        Channel::new_simple(SimpleChannelAction::NoOp), //todo: beam count
        Channel::new_simple(SimpleChannelAction::SpinLeft), //todo: spin left or right?
        Channel::new_simple(SimpleChannelAction::IntensityMasterDimmer),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Red))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Green))),
        Channel::new_simple(SimpleChannelAction::IntensityColor(Color::RGB(ColorRGB::Blue))),
        Channel::new_simple(SimpleChannelAction::NoOp), //todo: beam thickness
        Channel::new_simple(SimpleChannelAction::NoOp), //todo: length
        Channel::new_simple(SimpleChannelAction::Speed),
    ]),
));
//</editor-fold>

pub(crate) fn populate_fixture_store_defaults(){
    for fixture in [VRSL_PAR_LIGHT, VRSL_BAR_LIGHT, VRSL_BLINDER, VRSL_MOVING_HEAD, VRSL_LASER]{
        FIXTURE_STORE.put_path(fixture.get_path().as_ref(), fixture.clone());
    }
}