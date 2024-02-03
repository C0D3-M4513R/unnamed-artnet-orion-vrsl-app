use serde_derive::{Deserialize, Serialize};
use crate::artnet::universe::{UniverseChannels, UniverseDevices, Universes};

#[derive(Debug, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
pub struct CommonData{
    pub devices: Universes<UniverseDevices>,
    pub overrides: Universes<UniverseMasteredChannel<Option<u8>>>,
    pub global_multiplier: u8,
}

impl Default for CommonData{
    fn default() -> Self {
        CommonData {
            devices: Universes::default(),
            overrides: Universes::default(),
            global_multiplier: u8::MAX,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
pub struct UniverseMasteredChannel<T>{
    pub multiplier: u8,
    pub channels: UniverseChannels<T>
}

impl<T:Default> Default for UniverseMasteredChannel<T> {
    fn default() -> Self {
        UniverseMasteredChannel {
            multiplier: u8::MAX,
            channels: UniverseChannels::default(),
        }
    }
}