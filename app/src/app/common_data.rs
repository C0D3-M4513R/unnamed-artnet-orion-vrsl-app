use std::marker::PhantomData;
use std::ops::IndexMut;
use std::slice::Iter;
use serde_derive::{Deserialize, Serialize};
use crate::app::common_data::UniverseError::UniverseIdTooHigh;
use crate::artnet::fixture::MAX_UNIVERSE_ID;
use crate::artnet::universe::Universe;

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct CommonData{
    devices: Vec<Universe>,
    _mark: PhantomData<()>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, thiserror::Error)]
pub enum UniverseError{
    #[error("The id {0} is too high for a universe. It can be at maximum 32768.")]
    UniverseIdTooHigh(u16)
}

impl CommonData{
    fn ensure_size(&mut self, universe: u16){
        if universe as usize >= self.devices.len()  {
            let mut functions = {
                #[allow(clippy::cast_possible_truncation)]
                {
                    universe - self.devices.len() as u16 + 1
                }
            };
            self.devices.extend(core::iter::from_fn(||
                if functions == 0 {None}
                else {
                    let ret = Universe::new_default(functions);
                    functions-=1;
                    Some(ret)
                }
            ));
        }
    }
    pub fn create_or_get_universe(&mut self, universe: u16) -> Result<&mut Universe, UniverseError>{
        if universe > MAX_UNIVERSE_ID {return Err(UniverseIdTooHigh(universe));}
        self.ensure_size(universe);
        Ok(self.devices.index_mut(universe as usize))
    }
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
    pub fn device_iter(&self) -> Iter<'_, Universe> {
        self.devices.iter()
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Universe> {
        self.devices.get_mut(index)
    }

}