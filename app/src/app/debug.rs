use std::marker::PhantomData;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Debug{
    pub(super) debugging: bool,
    pub(super) debug_win_open: bool,
    _mark: PhantomData<()>,
}

impl Debug{
    pub(super) fn new_frame(&self){
        #[cfg(feature = "puffin")]
        {
            if self.debugging {
                puffin::GlobalProfiler::lock().new_frame();
            }
        }
    }
}