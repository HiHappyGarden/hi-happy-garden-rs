pub mod hardware;
pub mod gpio;

use core::ffi::c_char;
use osal_rs::os::types::ThreadHandle;

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



