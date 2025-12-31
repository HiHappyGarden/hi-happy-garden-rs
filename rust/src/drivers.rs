#![allow(unused_imports)]
mod button;
mod encoder;
pub(super) mod gpio; 

#[cfg(feature = "pico")]
mod pico;


#[cfg(feature = "pico")]
use crate::drivers::pico as plt;

pub mod platform {
    pub use crate::drivers::plt::gpio::*;
    pub use crate::drivers::plt::hardware::*;
    pub use crate::drivers::button::*;
    pub use crate::drivers::encoder::*;
}