use std::marker::PhantomData;
use std::sync::Arc;
use serde_derive::{Deserialize, Serialize};
use channel::Channel;

pub mod channel;
pub mod variables;

pub(crate) const MAX_UNIVERSE_ID:u16 = 1<<15;
pub(crate) const MAX_CHANNEL_ID:u16 = 1<<9;

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Fixture{
    manufacturer: Arc<str>,
    extra_path: Arc<[Arc<str>]>,
    model: Arc<str>,
    r#type: Arc<str>,
    channels: Arc<[Channel]>,
}

impl Fixture {
    #[inline]
    pub fn new(manufacturer: Arc<str>, model: Arc<str>, r#type: Arc<str>, channels: Arc<[Channel]>) -> Self {
        Self::new_path(
            manufacturer,
            Arc::from([]),
            model,
            r#type,
            channels,
        )
    }
    #[inline]
    pub const fn new_path(manufacturer: Arc<str>, extra_path: Arc<[Arc<str>]>, model: Arc<str>, r#type: Arc<str>, channels: Arc<[Channel]>) -> Self {
        Fixture {
            manufacturer,
            extra_path,
            model,
            r#type,
            channels,
        }
    }

    #[inline]
    pub const fn get_manufacturer(&self) -> &Arc<str> {
        &self.manufacturer
    }

    #[inline]
    pub const fn get_model(&self) -> &Arc<str> {
        &self.model
    }

    #[inline]
    pub const fn get_type(&self) -> &Arc<str> {
        &self.r#type
    }

    #[inline]
    pub const fn get_channels(&self) -> &Arc<[Channel]> {
        &self.channels
    }

    #[must_use]
    ///Get the path of this fixture in the fixture store.
    ///This is used by the fixture store, to place fixtures into the fixture store.
    pub fn get_path(&self) -> Arc<[Arc<str>]> {
        let mut path = Vec::with_capacity(2+self.extra_path.len());
        path.push(self.get_manufacturer().clone());
        path.extend_from_slice(self.extra_path.as_ref());
        path.push(self.get_model().clone());
        path.into()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Device {
    pub name: Arc<str>,
    ///`self.start_id + self.fixture.channels.len()` should always be inside an u9.
    pub start_id: u16,
    pub fixture: Fixture,
    _mark: PhantomData<()>,
}

impl Device{
    pub const fn new(name: Arc<str>, start_id: u16, fixture: Fixture) -> Self {
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
