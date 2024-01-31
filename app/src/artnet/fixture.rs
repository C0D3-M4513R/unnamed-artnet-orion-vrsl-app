use std::marker::PhantomData;
use std::sync::Arc;
use serde_derive::{Deserialize, Serialize};
use channel::Channel;

pub mod channel;

pub(crate) const MAX_UNIVERSE_ID:u16 = 2^15;
pub(crate) const MAX_CHANNEL_ID:u16 = 2^9;

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Fixture{
    pub manufacturer: Arc<str>,
    pub model: Arc<str>,
    pub r#type: Arc<str>,
    pub channels: Arc<[Channel]>,
}

impl Fixture {
    pub const fn new(manufacturer: Arc<str>, model: Arc<str>, r#type: Arc<str>, channels: Arc<[Channel]>) -> Self {
        Fixture {
            manufacturer,
            model,
            r#type,
            channels,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Device {
    pub name: Arc<str>,
    ///`self.start_id + self.fixture.channels.len()` should always be inside an u9.
    pub start_id: u16,
    pub fixture: Arc<Fixture>,
    _mark: PhantomData<()>,
}

impl Device{
    pub const fn new(name: Arc<str>, start_id: u16, fixture: Arc<Fixture>) -> Self {
        Device{
            name,
            start_id,
            fixture,
            _mark: PhantomData{},
        }
    }
    pub fn end_channel(&self) -> u64 {
        self.start_id as u64+self.fixture.channels.len() as u64
    }
}
