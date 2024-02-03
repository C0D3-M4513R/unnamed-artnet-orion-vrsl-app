use std::sync::Arc;
use serde_derive::{Deserialize, Serialize};
use channel::Channel;

pub mod channel;
pub mod variables;

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
        Self {
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
        path.into()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct Device {
    pub name: Arc<str>,
    ///`self.start_id + self.fixture.channels.len()` should always be inside an u9.
    start_id: ux2::u9,
    end_id: ux2::u9,
    pub fixture: Fixture,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, thiserror::Error)]
pub enum ChannelError{
    #[error("The id {0} is too high for a Channel. It can be at maximum 511.")]
    ChannelIdTooHigh(u16)
}

impl Device{
    pub fn new_u16(name: Arc<str>, start_id: u16, fixture: Fixture) -> Result<Self, ux2::TryFromIntError> {
        let u9_start_id = ux2::u9::try_from(start_id)?;
        Ok(Self{
            name,
            start_id: u9_start_id,
            end_id: ux2::u9::try_from(start_id as usize + fixture.channels.len())?,
            fixture,
        })
    }
    pub fn new(name: Arc<str>, start_id: ux2::u9, fixture: Fixture) -> Result<Self, ux2::TryFromIntError> {
        Ok(Self{
            name,
            start_id,
            end_id: ux2::u9::try_from(<ux2::u9 as Into<usize>>::into(start_id) + fixture.channels.len())?,
            fixture,
        })
    }

    #[inline]
    pub const fn start_channel(&self) -> ux2::u9 {
        self.start_id
    }

    #[inline]
    pub const fn end_channel(&self) -> ux2::u9 {
        self.end_id
    }
}
