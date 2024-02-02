use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::slice::Iter;
use serde_derive::{Deserialize, Serialize};
use crate::artnet::fixture::{Device};

#[derive(Debug, Clone, Default, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct UniverseChannelData<T>([[T;32];16]);
impl<T> UniverseChannelData<T>{
    #[allow(clippy::indexing_slicing)] //It can be proven, that this will NEVER panic
    fn get(&self, index: ux2::u9) -> &T {
        let index:u16 = index.into();
        &self.0[((index/32)&0x0F) as usize][(index%32) as usize]
    }
    #[allow(clippy::indexing_slicing)] //It can be proven, that this will NEVER panic
    fn get_mut(&mut self, index: ux2::u9) -> &mut T {
        let index:u16 = index.into();
        &mut self.0[((index/32)&0x0F) as usize][(index%32) as usize]
    }
}

#[allow(missing_copy_implementations)] //reason="The struct is too big"
#[derive(Debug, Clone, Default, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct UniverseChannels<T> {
    channels: UniverseChannelData<T>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, thiserror::Error)]
pub enum UniverseError{
    #[error("The id {0} is too high for a universe. It can be at maximum 32767.")]
    UniverseIdTooHigh(u16)
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Universes<T> {
    data: Vec<T>
}

impl<T:Default> Universes<T>{
    fn ensure_size(&mut self, universe: ux2::u15){
        self.data.resize_with(
            ux2::u15::max(
                universe,
                ux2::u15::try_from(self.data.len())
                    .unwrap_or(ux2::u15::MAX)
            ).into(),
            T::default
        );
    }
    pub fn create_or_get_universe(&mut self, universe: ux2::u15) -> &mut T {
        self.ensure_size(universe);
        self.data.index_mut(<ux2::u15 as Into<usize>>::into(universe))
    }
}
impl<T> Universes<T> {
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn iter(&self) -> Iter<'_, T> {
        self.data.iter()
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct UniverseDevices {
    devices: Vec<Device>,
    _marker: PhantomData<()>,
}

impl<'a> core::iter::IntoIterator for &'a UniverseDevices {
    type Item = &'a Device;
    type IntoIter = Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, thiserror::Error)]
pub enum InsertError {
    #[error("The starting Channel-Id is too big. {0} is bigger than 512")]
    StartIdTooBig(u16),
    #[error("The ending Channel-Id is too big. {0} is bigger than 512")]
    EndIdTooBig(u64),
    #[error("At least one Channel is already assigned in the requested channel-range.")]
    ChannelAlreadyAssigned
}

impl UniverseDevices {
    pub const fn new(devices: Vec<Device>) -> Self {
        Self{
            devices,
            _marker: PhantomData{}
        }
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
    pub fn iter(&self) -> Iter<Device> {
        self.devices.iter()
    }
    pub fn extend<I: IntoIterator<Item=Device>>(&mut self, iter: I) {
        self.devices.extend(iter)
    }
    pub fn remove(&mut self, index: usize) -> Device {
        self.devices.remove(index)
    }
    pub fn try_insert(&mut self, device: Device) -> Result<(), InsertError> {
        let start_channel = device.start_channel();
        let end_channel = device.end_channel();

        self.devices.sort_unstable_by_key(Device::start_channel);
        match self.devices.binary_search_by_key(&start_channel, Device::start_channel){
            Ok(_) => Err(InsertError::ChannelAlreadyAssigned),
            Err(v) => {
                //is there a previous element?
                if v > 0{
                    let prev_dev = self.devices.index(v-1);
                    if prev_dev.end_channel() >= start_channel {
                        return Err(InsertError::ChannelAlreadyAssigned);
                    }
                }
                //is there a next element?
                if v+1 < self.devices.len() {
                    let next_dev = self.devices.index(v+1);
                    if end_channel > next_dev.start_channel() {
                        return Err(InsertError::ChannelAlreadyAssigned);
                    }
                }
                self.devices.insert(v, device);
                Ok(())
            }
        }
    }
}