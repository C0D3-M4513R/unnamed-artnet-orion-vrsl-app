use serde_derive::{Deserialize, Serialize};
use crate::artnet::channel::Channel;

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Message {
    ///Register a channel override to value (`u8`)
    AddChannelOverride(Channel, u8),
    ///Remove a channel override
    RemoveChannelOverride(Channel),
    ///Search for ArtNetNodes again
    RescanArtNetNodes,
}