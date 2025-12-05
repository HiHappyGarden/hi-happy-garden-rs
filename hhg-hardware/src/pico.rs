use core::ffi::c_char;

// TaskHandle_t is a pointer to void in FreeRTOS
#[allow(non_camel_case_types)]
type TaskHandle_t = *mut core::ffi::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationMallocFailedHook() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationIdleHook() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationStackOverflowHook(_x_task: TaskHandle_t, _pc_task_name: *mut c_char) -> !{
    // Equivalent to: (void) xTask; (void) pcTaskName;
    // The underscore prefix already marks these as intentionally unused in Rust
    loop {}
}

