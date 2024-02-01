use std::sync::Arc;
use dashmap::{DashMap, DashSet};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use crate::artnet::fixture::channel::{Channel, ChannelAction, Color, ColorRGB, Range, SimpleChannelAction};
use crate::artnet::fixture::Fixture;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct FixtureStore{
    fixtures: DashSet<Arc<Fixture>>,
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

    fn put_path(&self, path: &[Arc<str>], fixture: Arc<Fixture>) {
        self.get_path(path, |fs|fs.fixtures.insert(fixture));
    }

    fn add_contained_fixtures(&self, ui: &mut egui::Ui,path: &mut Vec<Arc<str>>, item: &mut (Vec<Arc<str>>, Option<Arc<Fixture>>)) {
        let mut items = self.fixtures.iter().collect::<Vec<_>>();
        items.sort_by_cached_key(|x|x.key().model.clone());
        for i in items.into_iter(){
            let key = i.key();
            path.push(key.model.clone());
            if ui.selectable_label(
                item.1.as_ref().map(|f|f==key).unwrap_or(false),
                key.model.as_ref()
            ).clicked() {
                *item = (path.clone(), Some(key.clone()));
            }
            path.pop();
        }
    }
    pub fn _build_menu(&self, ui: &mut egui::Ui, path: &mut Vec<Arc<str>>, item: &mut (Vec<Arc<str>>, Option<Arc<Fixture>>)){
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
    pub fn build_menu(&self, ui: &mut egui::Ui, item: &mut (Vec<Arc<str>>, Option<Arc<Fixture>>)){
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
    VARYING, "varying",
    PAN180, "Pan180",
    PAN360, "Pan360",
    PAN540, "Pan540",
    TILT180, "Tilt180",
    TILT250, "Tilt250",
    TILT270, "Tilt270",
    STANDARD_MOVER_SPOTLIGHT, "Standard Mover Spotlight",
    MOVING_HEAD, "Moving Head",
    STANDARD_LASER, "Standard Laser",
    LASER, "Laser"
);
const VRSL_MOVING_HEAD:fn(usize, usize)->Arc<Fixture> = |max_pan, max_tilt|Arc::new(Fixture {
    manufacturer: VRSL.clone(),
    model: STANDARD_MOVER_SPOTLIGHT.clone(),
    r#type: MOVING_HEAD.clone(),
    channels: Arc::new([
        Channel::new_simple(SimpleChannelAction::PositionPan(max_pan)),
        //todo: vrsl - currently disabled, because the linear smoothing algorithm outweighs this
        Channel::new_simple(SimpleChannelAction::PositionPanFine(0)),
        Channel::new_simple(SimpleChannelAction::PositionTilt(max_tilt)),
        //todo: vrsl - currently disabled, because the linear smoothing algorithm outweighs this
        Channel::new_simple(SimpleChannelAction::PositionTiltFine(0)),
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
});
const VRSL_PAR_LIGHT:Lazy<Arc<Fixture>> = Lazy::new(||Arc::new(Fixture {
    manufacturer: VRSL.clone(),
    model: Arc::from("Standard Par Light"),
    r#type: Arc::from("Color Changer"),
    channels: Arc::new([
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
}));
const VRSL_BAR_LIGHT:Lazy<Arc<Fixture>> = Lazy::new(||Arc::new(Fixture {
    manufacturer: VRSL.clone(),
    model: Arc::from("Standard BarLight"),
    r#type: Arc::from("LED Bar (Pixels)"),
    channels: Arc::new([
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
}));
const VRSL_BLINDER:Lazy<Arc<Fixture>> = Lazy::new(||Arc::new(Fixture {
    manufacturer: VRSL.clone(),
    model: Arc::from("Standard Blinder"),
    r#type: Arc::from("Strobe"),
    channels: Arc::new([
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
}));
const VRSL_LASER:fn(usize, usize)->Arc<Fixture> = |max_pan, max_tilt|Arc::new(Fixture {
    manufacturer: VRSL.clone(),
    model: STANDARD_LASER.clone(),
    r#type: LASER.clone(),
    channels: Arc::new([
        Channel::new_simple(SimpleChannelAction::PositionPan(max_pan)), //todo: in qcl's fixture config, this is statically 0.
        Channel::new_simple(SimpleChannelAction::PositionTilt(max_tilt)), //todo: in qcl's fixture config, this is statically 0.
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
});
//</editor-fold>

const fn get_pan() -> [(usize, Lazy<Arc<str>, fn() -> Arc<str>>); 3] {
    [
        (180, PAN180),
        (360, PAN360),
        (540, PAN540)
    ]
}
const fn get_tilt() -> [(usize, Lazy<Arc<str>, fn() -> Arc<str>>); 3] {
    [
        (180, TILT180),
        (250, TILT250),
        (270, TILT270)
    ]
}

pub(crate) fn populate_fixture_store_defaults(){
    for fixture in [VRSL_PAR_LIGHT, VRSL_BAR_LIGHT, VRSL_BLINDER]{
        FIXTURE_STORE.put_path(&[fixture.manufacturer.clone()], fixture.clone());
    }
    for (pan, pan_key) in get_pan(){
        let pan_key = &*pan_key;
        for (tilt, tilt_key) in get_tilt(){
            let tilt_key = &*tilt_key;
            for fixture in [VRSL_MOVING_HEAD, VRSL_LASER] {
                let fixture = fixture(pan, tilt);
                FIXTURE_STORE.put_path(&[fixture.manufacturer.clone(), (&*VARYING).clone(), pan_key.clone(), tilt_key.clone()], fixture);
            }
        }
    }
}