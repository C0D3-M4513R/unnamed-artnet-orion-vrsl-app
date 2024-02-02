use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ChannelId {
    ///should only ever be an u15. Higher values will be silently ignored
    universe: ux2::u15,
    ///should only be an u9. Higher values will be silently ignored.
    channel: ux2::u9,
}