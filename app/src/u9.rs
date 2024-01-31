use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct U9(u16);

#[derive(Debug, thiserror::Error)]
pub enum U9ConvertError{
    #[error("The number {0} is too big to be converted to a U9.")]
    NumberTooBig(u16)
}

impl U9 {
    pub fn get(&self) -> u16 {
        self.0
    }
}
impl TryFrom<u16> for U9{
    type Error = U9ConvertError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0..=511 => {Ok(U9(value))}
            512..=u16::MAX => {Err(U9ConvertError::NumberTooBig(value))}
        }
    }
}
impl From<u8> for U9{
    fn from(value: u8) -> Self {
        U9(value as u16)
    }
}