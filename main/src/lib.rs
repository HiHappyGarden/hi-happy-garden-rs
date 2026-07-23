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
#![cfg_attr(feature = "tests", allow(dead_code))]

extern crate alloc;
extern crate osal_rs;
extern crate osal_rs_serde;
extern crate cjson_binding;

mod apps;
mod assets;
mod drivers;
mod traits;

const APP_TAG: &str = "main";

#[cfg(not(feature = "tests"))]
mod app {

    mod ffi {
        unsafe extern "C" {
            pub(super) fn print_systick_status();

            pub(super) fn get_g_setup_called() -> u32;
        }
    }

    use alloc::boxed::Box;

    use osal_rs::os::types::{StackType, TickType};
    use osal_rs::os::{System, SystemFn, ThreadFn, ThreadParam};
    use osal_rs::utils::Result;
    use osal_rs::log_fatal;

    use crate::APP_TAG;
    use crate::drivers::platform::Hardware;
    use crate::traits::state::Initializable;
    use ffi::{get_g_setup_called, print_systick_status};
    use crate::apps::AppMain;

    pub(super) const THREAD_NAME: &str = "main_trd";
    pub(super) const STACK_SIZE: StackType = 1_024*8; // 8KB stack

    static mut HARDWARE: Option<Hardware> = None;
    static mut APP_MAIN: Option<AppMain> = None;


    pub(super) fn main_thread(_thread: Box<dyn ThreadFn>, _: Option<ThreadParam>) -> Result<ThreadParam>{
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


}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() {
    osal_rs::log::set_enable_color(true);

    #[cfg(not(feature = "tests"))]
    {
        use osal_rs::os::{System, SystemFn, Thread, ThreadFn};
        use crate::app::{STACK_SIZE, THREAD_NAME, main_thread};
        use crate::drivers::platform::ThreadPriority;



        let mut thread = Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal);
        let _ = match thread.spawn(None, main_thread) {
            
            Ok(spawned) =>  {
                use osal_rs::log_info;

                log_info!(APP_TAG, "Start main thread\r\n");
                spawned
            }
            Err(e) => panic!("Failed to spawn main thread: {:?}", e)

        };

        System::start();
    }

    #[cfg(feature = "tests")]
    {
        use osal_rs::os::System;
        use crate::osal_rs::os::SystemFn;

        perform_tests();

        System::start();
    }
}



#[cfg(feature = "tests")]
fn perform_tests() {

    match osal_rs_tests::freertos::run_all_tests() {
        Ok(_) => osal_rs::log_info!(APP_TAG, "All tests passed!"),
        Err(e) => panic!("Tests failed with error: {:?}", e)
    };

}



