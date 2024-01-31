use std::sync::Arc;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Channel{
    action: ChannelAction,
}

impl Channel{
    pub fn new(action: ChannelAction) -> Self {
        Channel{
            action
        }
    }
    pub fn new_simple(action: SimpleChannelAction) -> Self {
        Channel{
            action: ChannelAction::SimpleChannelAction(action)
        }
    }

}
///The contained data represents one distinct range of a channel.
///
/// `continuous` should be true, if different values in the range have a different effect.
/// If different values in the range have the same effect, it should be false.
///
/// `start` and `end` define the range (both inclusive) where going
/// from start to end will increase the `action`.
///
/// Please note that it should be expected, that `start<end` is possible and should be respected.
/// In this case the lower value is the start of the range in the actual dmx output range,
/// but the `action` gets stronger with a lower value.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Range{
    continuous: bool,
    start: u8,
    end: u8,
    action: SimpleChannelAction
}

impl Range{
    pub fn new(continuous: bool, start: u8, end: u8, action: SimpleChannelAction) -> Self{
        Range{
            continuous,
            start,
            end,
            action,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ChannelAction{
    SimpleChannelAction(SimpleChannelAction),
    ///The people instantiating this are responsible for putting sensible data in here.
    Selection(Arc<[Range]>)
}

///What does this channel Control?
///In general, it is assumed, that a higher dmx value will lead to a higher action.
///If that is not the case a `ChannelAction::Selection` should be used to create an inverse map.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum SimpleChannelAction{
    NoOp,
    ///data is total range of this channel in degrees
    PositionPan(usize),
    ///data is total range of this channel in degrees
    PositionPanFine(usize),
    ///data is total range of this channel in degrees
    PositionTilt(usize),
    ///data is total range of this channel in degrees
    PositionTiltFine(usize),
    Speed,
    Strobo,
    SpinRight,
    SpinLeft,
    GOBOSelection,
    BeamZoom,
    IntensityMasterDimmer,
    IntensityColor(Color),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Color{
    RGB(ColorRGB),
    HSV(ColorHSV),
    HSL(ColorHSL),
    HSI(ColorHSI),
}
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ColorRGB{ Red, Green, Blue }
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ColorHSV{ Hue, Saturation, Value }
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ColorHSL{ Hue, Saturation, Lightness }
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ColorHSI{ Hue, Saturation, Intensity }