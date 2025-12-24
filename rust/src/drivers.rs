#[cfg(feature = "pico")]
mod pico;

#[cfg(feature = "pico")]
use crate::drivers::pico as plt;

pub mod platform {
    pub use crate::drivers::plt::*;
}