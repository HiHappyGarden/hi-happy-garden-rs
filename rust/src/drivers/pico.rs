#![allow(unused_imports)]

pub(super) mod hardware;
pub(super) mod gpio;

use core::ffi::c_char;
use osal_rs::os::types::ThreadHandle;

pub use crate::drivers::pico::gpio::*;
pub use crate::drivers::pico::hardware::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationMallocFailedHook() -> ! {
    // Hook for malloc failures - hang here for debugging
    #[allow(clippy::empty_loop)]
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationIdleHook() {
    // Idle hook - can be used for low power modes
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationStackOverflowHook(_x_task: ThreadHandle, _pc_task_name: *mut c_char) -> ! {
    // Stack overflow detected - hang here for debugging
    #[allow(clippy::empty_loop)]
    loop {}
}



