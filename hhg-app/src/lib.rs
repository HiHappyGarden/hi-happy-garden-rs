#![no_std]

extern crate alloc;
extern crate osal_rs;

#[unsafe(no_mangle)]
pub extern "C" fn app_add(left: u64, right: u64) -> u64 {
    left + right
}
