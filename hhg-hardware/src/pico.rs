mod hardware;

use core::ffi::c_char;
use osal_rs::os::types::ThreadHandle;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationMallocFailedHook() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationIdleHook() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationStackOverflowHook(_x_task: ThreadHandle, _pc_task_name: *mut c_char) -> !{
    loop {}
}



