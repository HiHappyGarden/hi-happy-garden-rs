#![no_std]

extern crate alloc;
extern crate osal_rs;

#[cfg(feature = "pico")]
pub mod pico;

#[cfg(feature = "pico")]
use crate::pico as platform;

#[unsafe(no_mangle)]
pub extern "C" fn hw_add(left: u64, right: u64) -> u64 {
    left + right
}

