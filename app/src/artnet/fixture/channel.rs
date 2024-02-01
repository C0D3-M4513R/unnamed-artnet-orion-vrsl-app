use std::sync::Arc;
use serde_derive::{Deserialize, Serialize};
use crate::artnet::fixture::variables::{VariableChannelAction, VariableSelection};

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Channel{
    action: Action,
}

impl Channel{
    pub const fn new(action: Action) -> Self {
        Self{
            action
        }
    }
    pub const fn new_simple(action: SimpleAction) -> Self {
        Self::new(Action::SimpleAction(action))
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
#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Range{
    continuous: bool,
    start: u8,
    end: u8,
    action: SimpleAction
}

impl Range{
    pub const fn new(start: u8, end: u8, action: SimpleAction) -> Self{
        Self::with_continuous(action.is_continuous(), start, end, action)
    }

    pub const fn with_continuous(continuous: bool, start: u8, end: u8, action: SimpleAction) -> Self{
        Self{
            continuous,
            start,
            end,
            action,
        }
    }

    #[inline]
    pub const fn is_continuous(&self) -> bool {
        self.continuous || self.action.is_continuous()
    }

    #[inline]
    pub const fn is_inverted(&self) -> bool {
        self.start>self.end
    }

    #[inline]
    pub const fn get_start(&self) -> u8 {
        if self.is_inverted(){
            self.end
        }else {
            self.start
        }
    }

    #[inline]
    pub const fn get_end(&self) -> u8 {
        if self.is_inverted(){
            self.start
        }else {
            self.end
        }
    }

    #[inline]
    ///How many distinct values are in this range?
    ///This will at maximum be 256.
    pub const fn len(&self) -> u16 {
        //+1 to the end, because otherwise e.g. 255-0 = 255, but there are 256 contained values
        self.get_end() as u16 + 1 - self.get_start() as u16
    }

    pub fn scale_to_range(&self, input:u64, variable_selection: VariableSelection) -> u8 {
        if self.action.auto_conversion() {
            self.action.scale_to_range(input, variable_selection)
        }
        else if !self.is_continuous() {
            //if we are not continuous try to pick a value, that's as far away from other values as possible.
            //if the start or end of the range is 0 or 255 respectively, we take that (since you cannot go higher/lower).
            if self.get_start() == u8::MIN { u8::MIN }
            else if self.get_end() == u8::MAX { u8::MAX  }
            //else we just take the middle value, which is just the value in the middle of start and end.
            else {
                #[allow(clippy::cast_possible_truncation)]
                {
                    self.get_start() + (self.len()/2) as u8
                }
            }
        }
        else {
            #[allow(clippy::cast_possible_truncation)]
            {
                //self.len()/256 is basically the scaling factor.
                //input has a range of 256 values, output has a range of self.len()
                //input/input_range*output_range <=> input*output_range/input_range
                //
                //please note that we do not use the alternative of input*(output_range/input_range)
                //because that would entail floating point arithmetic, which is imprecise (and thus not const).
                //
                //then we just add self.get_start(), to make the value actually fall within the range we need.
                self.get_start() + (u128::from(input) * u128::from(self.len()) / 256) as u8
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Action {
    SimpleAction(SimpleAction),
    ///The people instantiating this are responsible for putting sensible data in here.
    Selection(Arc<[Range]>)
}

///What does this channel Control?
///In general, it is assumed, that a higher dmx value will lead to a higher action.
///If that is not the case a `ChannelAction::Selection` should be used to create an inverse map.
#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum SimpleAction {
    NoOp,
    VariableChannelAction(VariableChannelAction),
    Speed,
    Strobo,
    SpinRight,
    SpinLeft,
    GOBOSelection,
    BeamZoom,
    IntensityMasterDimmer,
    IntensityColor(Color),
}

impl SimpleAction {
    ///True, if the channel action converts the dmx value in some way.
    pub const fn auto_conversion(&self) -> bool {
        matches!(self, Self::VariableChannelAction(_))
    }
    ///True, if different values produce a different effect
    pub const fn is_continuous(&self) -> bool {
        !matches!(self, Self::GOBOSelection | Self::NoOp)
    }

    #[allow(clippy::cast_possible_truncation)] //yes, we want this here
    pub fn scale_to_range(&self, input: u64, variable_selection: VariableSelection) -> u8 {
        match self{
            Self::VariableChannelAction(var)
                => var.resolve(variable_selection).scale_to_range(input) as u8,
            Self::NoOp |
            Self::Speed |
            Self::Strobo |
            Self::SpinRight |
            Self::SpinLeft |
            Self::GOBOSelection |
            Self::BeamZoom |
            Self::IntensityMasterDimmer |
            Self::IntensityColor(_)
                => input as u8
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Color{
    Rgb(ColorRGB),
    Hsv(ColorHSV),
    Hsl(ColorHSL),
    Hsi(ColorHSI),
}
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ColorRGB{ Red, Green, Blue }
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ColorHSV{ Hue, Saturation, Value }
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ColorHSL{ Hue, Saturation, Lightness }
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ColorHSI{ Hue, Saturation, Intensity }