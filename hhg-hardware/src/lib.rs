#![no_std]

extern crate alloc;
extern crate osal_rs;

use osal_rs::os::types::TickType;
use osal_rs::os::config::{CPU_CLOCK_HZ, TICK_RATE_HZ};

#[cfg(feature = "pico")]
pub mod pico;

#[cfg(feature = "pico")]
use crate::pico as platform;

#[cfg(feature = "pico")]
pub use platform::*;

#[unsafe(no_mangle)]
pub extern "C" fn hw_add(left: u64, right: u64) -> u64 {
    let _tick: TickType = 5;
    // Example of using config constants
    let _cpu_freq = CPU_CLOCK_HZ;
    let _tick_rate = TICK_RATE_HZ;
    left + right
}
