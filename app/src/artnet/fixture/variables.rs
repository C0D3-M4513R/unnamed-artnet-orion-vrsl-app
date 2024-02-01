use std::sync::Arc;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Variable<T>{
    Set(T),
    Selection(Arc<[T]>),
}