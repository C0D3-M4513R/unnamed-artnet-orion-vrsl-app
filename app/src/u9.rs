#![allow(dead_code)] //todo: re-check, once the major implementation of this app has been done.
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct U9(u16);

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("The number {0} is too big to be converted to a U9.")]
    NumberTooBig(u16)
}

impl U9 {
    pub const fn get(self) -> u16 {
        self.0
    }
}
impl TryFrom<u16> for U9{
    type Error = ConvertError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0..=511 => {Ok(Self(value))}
            512..=u16::MAX => {Err(ConvertError::NumberTooBig(value))}
        }
    }
}
impl From<u8> for U9{
    fn from(value: u8) -> Self {
        Self(u16::from(value))
    }
}