use std::sync::Arc;
use serde_derive::{Deserialize, Serialize};
use crate::degree::scale_deg;

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Variable<T>{
    Set(T),
    Selection(Arc<[T]>, T),
}

impl<T> Variable<T> {
    fn select(&self, index: Option<usize>) -> &T {
        // (&[0u8] as &[u8])[0];
        // Box::leak()
        match self{
            Self::Set(x) => x,
            Self::Selection(val, def) =>
                index.map_or(def, |index| val.get(index).unwrap_or(def)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum VariableChannelAction{
    ///data is total range of this channel in micro-arc-seconds
    PositionPan(Variable<u64>),
    ///data is total range of this channel in micro-arc-seconds
    PositionPanFine(Variable<u64>),
    ///data is total range of this channel in micro-arc-seconds
    PositionTilt(Variable<u64>),
    ///data is total range of this channel in micro-arc-seconds
    PositionTiltFine(Variable<u64>),
}
impl VariableChannelAction{
    pub fn resolve(&self, variable_selection: VariableSelection) -> ResolvedVariableChannelAction{
        match self {
            Self::PositionPan(v) => ResolvedVariableChannelAction::PositionPan(*v.select(variable_selection.pan)),
            Self::PositionPanFine(v) => ResolvedVariableChannelAction::PositionPanFine(*v.select(variable_selection.panfine)),
            Self::PositionTilt(v) => ResolvedVariableChannelAction::PositionTilt(*v.select(variable_selection.tilt)),
            Self::PositionTiltFine(v) => ResolvedVariableChannelAction::PositionTiltFine(*v.select(variable_selection.tiltfine)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum ResolvedVariableChannelAction{
    ///data is total range of this channel in degrees / self.1
    /// (e.g. if self.1 = 60 then the variable is in 1/60 degrees aka. arc-minutes)
    PositionPan(u64),
    ///data is total range of this channel in degrees * self.1
    /// (e.g. if self.1 = 60 then the variable is in 1/60 degrees aka. arc-minutes)
    PositionPanFine(u64),
    ///data is total range of this channel in degrees * self.1
    /// (e.g. if self.1 = 60 then the variable is in 1/60 degrees aka. arc-minutes)
    PositionTilt(u64),
    ///data is total range of this channel in degrees * self.1
    /// (e.g. if self.1 = 60 then the variable is in 1/60 degrees aka. arc-minutes)
    PositionTiltFine(u64),
}

impl ResolvedVariableChannelAction {
    pub const fn scale_to_range(&self, input: u64) -> u64 {
        match self {
            Self::PositionPan(range) |
            Self::PositionPanFine(range) |
            Self::PositionTilt(range) |
            Self::PositionTiltFine(range)
            =>  scale_deg(*range, input, 256),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct VariableSelection{
    pan: Option<usize>,
    panfine: Option<usize>,
    tilt: Option<usize>,
    tiltfine: Option<usize>,
}

impl VariableSelection{
    pub const fn new(pan: Option<usize>, panfine: Option<usize>, tilt: Option<usize>, tiltfine: Option<usize>) -> Self {
        Self{
            pan,
            panfine,
            tilt,
            tiltfine,
        }
    }
}