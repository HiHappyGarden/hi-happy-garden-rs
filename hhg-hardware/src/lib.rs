#![no_std]

extern crate alloc;
extern crate osal_rs;

#[cfg(feature = "pico")]
pub mod pico;

#[cfg(feature = "pico")]
use crate::pico as platform;

#[cfg(feature = "pico")]
pub use platform::*;
