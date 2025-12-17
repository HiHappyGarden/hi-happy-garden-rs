
use osal_rs::os::{System, SystemFn, Thread, ThreadFn};

unsafe extern "C" {
    fn printf(format: *const u8, ...) -> i32;
}

#[cfg(feature = "examples")]
extern "C" {
    fn xSemaphoreCreateRecursiveMutex() -> *mut core::ffi::c_void;
    fn xQueueTakeMutexRecursive(mutex: *mut core::ffi::c_void, ticks: u32) -> i32;
    fn xQueueGiveMutexRecursive(mutex: *mut core::ffi::c_void) -> i32;
}

#[cfg(feature = "examples")]
const PD_TRUE: i32 = 1;
#[cfg(feature = "examples")]
const PORT_MAX_DELAY: u32 = 0xFFFFFFFF;

#[cfg(feature = "examples")]
static mut EXAMPLES_THREAD: Option<Thread> = None;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_main() {

    printf(b"Starting RTOS...\n\0".as_ptr());
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_start_os() {
    System::start();
}
