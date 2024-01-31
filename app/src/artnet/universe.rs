use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::slice::Iter;
use serde_derive::{Deserialize, Serialize};
use crate::artnet::fixture::Device;

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct Universe{
    pub id: u16,
    devices: Vec<Device>,
    _marker: PhantomData<()>,
}

impl Index<u16> for Universe {
    type Output = Device;

    fn index(&self, index: u16) -> &Self::Output {
        self.devices.index(index as usize)
    }
}

impl IndexMut<u16> for Universe{
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.devices.index_mut(index as usize)
    }
}

impl<'a> core::iter::IntoIterator for &'a Universe {
    type Item = &'a Device;
    type IntoIter = Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, thiserror::Error)]
pub enum UniverseInsertError {
    #[error("The starting Channel-Id is too big. {0} is bigger than 512")]
    StartIdTooBig(u16),
    #[error("The ending Channel-Id is too big. {0} is bigger than 512")]
    EndIdTooBig(u64),
    #[error("At least one Channel is already assigned in the requested channel-range.")]
    ChannelAlreadyAssigned
}

impl Universe{
    pub const fn new(id: u16, devices: Vec<Device>) -> Self {
        Universe{
            id,
            devices,
            _marker: PhantomData{}
        }
    }
    pub const fn new_default(id: u16) -> Self {
        Universe{
            id,
            devices: Vec::new(),
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
    pub fn try_insert(&mut self, device: Device) -> Result<(), UniverseInsertError> {
        if device.start_id >= 512{
            return Err(UniverseInsertError::StartIdTooBig(device.start_id));
        }
        let end_channel = device.end_channel();
        if end_channel >= 512 {
            return Err(UniverseInsertError::EndIdTooBig(end_channel));
        }
        let end_channel = end_channel as u16;
        self.devices.sort_unstable_by_key(|device|device.start_id);
        match self.devices.binary_search_by_key(&device.start_id, |other_dev|other_dev.start_id){
            Ok(_) => Err(UniverseInsertError::ChannelAlreadyAssigned),
            Err(v) => {
                //is there a previous element?
                if v > 0{
                    let prev_dev = self.devices.index(v-1);
                    if prev_dev.end_channel() >= device.start_id as u64 {
                        return Err(UniverseInsertError::ChannelAlreadyAssigned);
                    }
                }
                //is there a next element?
                if v+1 < self.devices.len() {
                    let next_dev = self.devices.index(v+1);
                    if end_channel > next_dev.start_id {
                        return Err(UniverseInsertError::ChannelAlreadyAssigned);
                    }
                }
                self.devices.insert(v, device);
                Ok(())
            }
        }
    }
}