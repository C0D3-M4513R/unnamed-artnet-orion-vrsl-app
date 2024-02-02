use std::marker::PhantomData;
use std::slice::Iter;
use serde_derive::{Deserialize, Serialize};
use crate::artnet::universe::{UniverseDevices, Universes};

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct CommonData{
    devices: Universes<UniverseDevices>,
    _mark: PhantomData<()>,
}

impl CommonData{
    pub fn create_or_get_universe(&mut self, universe: ux2::u15) -> &mut UniverseDevices{
        self.devices.create_or_get_universe(universe)
    }
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
    pub fn device_iter(&self) -> Iter<'_, UniverseDevices> {
        self.devices.iter()
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut UniverseDevices> {
        self.devices.get_mut(index)
    }

}