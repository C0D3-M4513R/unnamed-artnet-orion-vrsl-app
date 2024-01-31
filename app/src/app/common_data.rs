use serde_derive::{Deserialize, Serialize};
use crate::artnet::fixture::{Device, Fixture};

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct CommonData{
    fixtures: Vec<Fixture>,
    devices: Vec<Device>,
}