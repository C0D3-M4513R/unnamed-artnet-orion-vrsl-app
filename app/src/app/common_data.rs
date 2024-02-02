use serde_derive::{Deserialize, Serialize};
use crate::artnet::universe::{UniverseChannels, UniverseDevices, Universes};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommonData{
    pub devices: Universes<UniverseDevices>,
    pub overrides: Universes<(u8, UniverseChannels<Option<u8>>)>,
    pub global_override: Option<u8>,
}