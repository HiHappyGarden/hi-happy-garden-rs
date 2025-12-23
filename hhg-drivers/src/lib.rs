#![no_std]

extern crate alloc;
extern crate osal_rs;


#[cfg(feature = "pico")]
pub mod pico;

use crate::pico::hardware::{self, Hardware};
#[cfg(feature = "pico")]
use crate::pico as plt;

#[cfg(feature = "pico")]
pub mod platform {
    pub use crate::plt::*;    
}


use alloc::boxed::Box;

use osal_rs::os::types::TickType;
use osal_rs::os::{System, SystemFn, Thread, ThreadFn, ThreadParam};
use osal_rs::log::set_enable_color;
use osal_rs::utils::Result;
use osal_rs::{log_info};

use crate::ffi::{get_g_setup_called, print_systick_status};

mod ffi {
    unsafe extern "C" {
        pub(super) fn print_systick_status();

        pub(super) fn get_g_setup_called() -> u32;
    }
}

const APP_TAG: &str = "hhg-drivers";

static mut HARDWARE_THREAD: Option<Thread> = None;

//  #[cfg(not(feature = "tests"))]
fn hardware_main_thread(_thread: Box<dyn ThreadFn>, _param: Option<ThreadParam>) -> Result<ThreadParam>{
    unsafe {
        loop {
            if get_g_setup_called() == 1 {
                break;
            }
        }

        print_systick_status();
    }
    
    log_info!(APP_TAG, "Initial tick count: {}", System::get_tick_count());
    
    log_info!(APP_TAG, "Start ...");

    let mut hardware = Hardware::new();

    
    
    loop {
        System::delay(TickType::MAX);
    }

    panic!();
}




#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() {
    set_enable_color(false);

    #[cfg(not(feature = "tests"))]
    {
        match Thread::new("hardware_main_thread", 4096, 3, hardware_main_thread).spawn(None) {
            Ok(spawned) =>  unsafe {HARDWARE_THREAD = Some(spawned)},
            Err(e) => panic!("Failed to spawn hardware_main_thread: {:?}", e)
        };
    }

    #[cfg(feature = "tests")]
    {
        perform_tests();
    }
}





#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_os() {
    System::start();
}



#[cfg(feature = "tests")]
fn perform_tests() {


    log_info!(APP_TAG, "Creating osal rs test thread...");

    match Thread::new("osal_rs_test", 4096, 3, Box::new(|_, _| {
        use osal_rs::utils::Error;


        match osal_rs_tests::freertos::run_all_tests() {
            Ok(_) => log_info!(APP_TAG, "All tests passed!"),
            Err(e) => panic!("Tests failed with error: {:?}", e)
        };

        Err(Error::Unhandled(""))
    })).spawn(None) {
        Ok(_spawned) =>  log_info!(APP_TAG, "Thread spawned successfully!"),
        Err(e) => panic!("Failed to spawn osal rs test thread: {:?}", e)
    };
}



