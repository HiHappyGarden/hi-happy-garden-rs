/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/
 
 #![no_std]

extern crate alloc;
extern crate osal_rs;
extern crate osal_rs_serde;
extern crate cjson_binding;

mod apps;
mod assets;
mod drivers;
mod traits;


mod ffi {
    unsafe extern "C" {
        pub(super) fn print_systick_status();

        pub(super) fn get_g_setup_called() -> u32;
    }
}


use alloc::boxed::Box;

use osal_rs::os::types::{StackType, TickType};
use osal_rs::os::{System, SystemFn, Thread, ThreadFn, ThreadParam};
use osal_rs::log::set_enable_color;
use osal_rs::utils::Result;
use osal_rs::{log_fatal, log_info};

use crate::drivers::platform::Hardware;
use crate::traits::state::Initializable;
use crate::ffi::{get_g_setup_called, print_systick_status};
use crate::apps::AppMain;

const APP_TAG: &str = "rust";
const THREAD_NAME: &str = "main_trd";
const STACK_SIZE: StackType = 1_024*5; // 5KB stack

static mut HARDWARE: Option<Hardware> = None;
static mut APP_MAIN: Option<AppMain> = None;


#[cfg(not(feature = "tests"))]
fn main_thread(_thread: Box<dyn ThreadFn>, _param: Option<ThreadParam>) -> Result<ThreadParam>{
    use osal_rs::log_debug;


    unsafe {
        loop {
            if get_g_setup_called() == 1 {
                break;
            }
        }

        print_systick_status();
    }

    #[cfg(debug_assertions)]
    log_debug!(APP_TAG, "OUT_DIR: {}", env!("OUT_DIR"));
    

    log_debug!(APP_TAG, "Initial tick count: {}", System::get_tick_count());

    unsafe {
        HARDWARE = Some(Hardware::new()); 

        let hardware = &mut *&raw mut HARDWARE;

        let hardware = match hardware {
            Some(hardware) => hardware,
            None => panic!("No memory for hardware"),
        };

        if let Err(err) = hardware.init() {
            log_fatal!(APP_TAG, "Hardware error: {:?}", err);
            panic!("Hardware initialization failed");
        }

        APP_MAIN = Some(AppMain::new(hardware));

        let app = &mut *&raw mut APP_MAIN;

        let app = match app {
            Some(app) => app,
            None => panic!("No memory for app main"),
        };

        if let Err(err) = app.init() {
            log_fatal!(APP_TAG, "App error: {:?}", err);
            panic!("App initialization failed");
        }

    }

    

    loop {
        System::delay(TickType::MAX);
    }
}




#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() {
    set_enable_color(true);

    #[cfg(not(feature = "tests"))]
    {
        use crate::drivers::platform::ThreadPriority;

        let mut thread = Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal);
        let _ = match thread.spawn(None, main_thread) {
            
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



