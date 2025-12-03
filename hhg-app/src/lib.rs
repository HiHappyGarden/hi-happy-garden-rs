#![no_std]

// Il panic handler è fornito da hhg-hardware
extern crate hhg_hardware;

#[unsafe(no_mangle)]
pub extern "C" fn app_add(left: u64, right: u64) -> u64 {
    left + right
}
