#![no_std]

extern crate alloc;
extern crate osal_rs;

mod app;
mod drivers;
mod traits;


mod ffi {
    unsafe extern "C" {
        pub(super) fn print_systick_status();

        pub(super) fn get_g_setup_called() -> u32;
    }
}


use alloc::boxed::Box;

use osal_rs::os::types::TickType;
use osal_rs::os::{System, SystemFn, Thread, ThreadFn, ThreadParam};
use osal_rs::log::set_enable_color;
use osal_rs::utils::Result;
use osal_rs::{log_fatal, log_info};

use crate::drivers::platform::Hardware;
use crate::traits::state::Initializable;
use crate::ffi::{get_g_setup_called, print_systick_status};


const APP_TAG: &str = "rust";



#[cfg(not(feature = "tests"))]
fn main_thread(_thread: Box<dyn ThreadFn>, _param: Option<ThreadParam>) -> Result<ThreadParam>{
    unsafe {
        loop {
            if get_g_setup_called() == 1 {
                break;
            }
        }

        print_systick_status();
    }
    log_info!(APP_TAG, "Initial tick count: {}", System::get_tick_count());
    
    let mut hardware = Hardware::new();

    if let Err(err) = hardware.init() {
        log_fatal!(APP_TAG, "Hardware error: {:?}", err);
    }

    loop {
        System::delay(TickType::MAX);
    }
}




#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() {
    set_enable_color(false);

    #[cfg(not(feature = "tests"))]
    {
        let mut thread = Thread::new("main_trd", 4096, 3);
        let _thread = match thread.spawn(None, main_thread) {
            Ok(spawned) =>  {
                log_info!(APP_TAG, "Start main thread\r\n");
                spawned
            }
            Err(e) => panic!("Failed to spawn main_trd: {:?}", e)
        };

        System::start();
    }

    #[cfg(feature = "tests")]
    {
        perform_tests();
    }
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



