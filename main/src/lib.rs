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

mod ffi {
    unsafe extern "C" {
        pub(crate) fn print_systick_status();

        pub(crate) fn get_g_setup_called() -> u32;
    }
}

#[cfg(not(feature = "tests"))]
mod app {

    use alloc::boxed::Box;

    use osal_rs::os::types::{StackType, TickType};
    use osal_rs::os::{System, SystemFn, ThreadFn, ThreadParam};
    use osal_rs::utils::Result;
    use osal_rs::log_fatal;

    use crate::APP_TAG;
    use crate::drivers::platform::Hardware;
    use crate::traits::state::Initializable;
    use crate::ffi::{get_g_setup_called, print_systick_status};
    use crate::apps::AppMain;

    pub(super) const THREAD_NAME: &str = "main_thr";
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

            osal_rs::log_info!(APP_TAG, "App initialized heap_free:{}", System::get_free_heap_size());

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
        use osal_rs::os::{System, SystemFn, Thread, ThreadFn};
        use crate::drivers::platform::ThreadPriority;
        use crate::tests::{TEST_STACK_SIZE, TEST_THREAD_NAME, test_thread};

        // The test suite must run with the scheduler already running: blocking
        // calls (`EventGroup::wait`, `Queue::receive`, ...) yield through the
        // port layer, which dereferences `pxCurrentTCB`. Called straight from
        // `main()` that pointer is still NULL and the yield faults, so the
        // tests are spawned as a task and only entered once `System::start()`
        // has handed control to the scheduler.
        let mut thread = Thread::new_with_to_priority(TEST_THREAD_NAME, TEST_STACK_SIZE, ThreadPriority::Normal);
        if let Err(e) = thread.spawn(None, test_thread) {
            panic!("Failed to spawn test thread: {:?}", e);
        }

        System::start();
    }
}


/// On-target firmware test suite.
///
/// `cargo test` cannot run on a `no_std` target, so the suite is linked into
/// the firmware behind the `tests` feature (CMake `-DHHG_TESTS=ON`) and runs
/// on the real board in place of `AppMain`: first the osal-rs suite, then the
/// firmware tests of `drivers` and `apps`.
///
/// Every module under test owns a child `tests` module (the same layout as a
/// `#[cfg(test)] mod tests`), so the tests can reach `pub(in ...)` and private
/// items without widening their visibility.
///
/// A test is a plain `fn() -> Result<()>`: the [`test_assert!`] /
/// [`test_assert_eq!`] macros return an `Err` instead of panicking, so a
/// failing test is logged and counted while the rest of the suite keeps
/// running — reflashing the board for each failure would be far too slow.
#[cfg(feature = "tests")]
mod tests {

    use alloc::boxed::Box;

    use osal_rs::os::types::{StackType, TickType};
    use osal_rs::os::{System, SystemFn, ThreadFn, ThreadParam};
    use osal_rs::utils::Result;
    use osal_rs::{log_error, log_fatal, log_info};

    use crate::APP_TAG;
    use crate::drivers::platform::Hardware;
    use crate::ffi::get_g_setup_called;
    use crate::traits::state::Initializable;

    pub(super) const TEST_THREAD_NAME: &str = "test_thr";
    pub(super) const TEST_STACK_SIZE: StackType = 1_024 * 8; // 8KB stack

    static mut HARDWARE: Option<Hardware> = None;

    /// Returns `Err` with file/line and the failed condition when `$cond` is false.
    macro_rules! test_assert {
        ($cond:expr) => {
            if !($cond) {
                return Err(osal_rs::utils::Error::UnhandledOwned(alloc::format!(
                    "{}:{}: assertion failed: {}", file!(), line!(), stringify!($cond)
                )));
            }
        };
        ($cond:expr, $($arg:tt)+) => {
            if !($cond) {
                return Err(osal_rs::utils::Error::UnhandledOwned(alloc::format!(
                    "{}:{}: {}", file!(), line!(), format_args!($($arg)+)
                )));
            }
        };
    }
    pub(crate) use test_assert;

    /// Returns `Err` with file/line and both values when `$left != $right`.
    macro_rules! test_assert_eq {
        ($left:expr, $right:expr) => {
            match (&$left, &$right) {
                (left, right) => {
                    if *left != *right {
                        return Err(osal_rs::utils::Error::UnhandledOwned(alloc::format!(
                            "{}:{}: {} != {} ({:?} != {:?})",
                            file!(), line!(), stringify!($left), stringify!($right), left, right
                        )));
                    }
                }
            }
        };
        ($left:expr, $right:expr, $($arg:tt)+) => {
            match (&$left, &$right) {
                (left, right) => {
                    if *left != *right {
                        return Err(osal_rs::utils::Error::UnhandledOwned(alloc::format!(
                            "{}:{}: {} ({:?} != {:?})",
                            file!(), line!(), format_args!($($arg)+), left, right
                        )));
                    }
                }
            }
        };
    }
    pub(crate) use test_assert_eq;

    /// Runs each test function, logging and recording its outcome in `$stats`.
    ///
    /// ```ignore
    /// run_tests!(TAG, stats; test_a, test_b);
    /// ```
    macro_rules! run_tests {
        ($tag:expr, $stats:expr; $($test:ident),+ $(,)?) => {
            osal_rs::log_info!($tag, "========== Running {} ==========", $tag);
            $( $stats.record($tag, stringify!($test), $test()); )+
        };
    }
    pub(crate) use run_tests;

    /// Pass/fail counters of the whole firmware suite.
    #[derive(Debug, Default)]
    pub(crate) struct TestStats {
        pub(crate) passed: u32,
        pub(crate) failed: u32,
    }

    impl TestStats {
        pub(crate) fn record(&mut self, tag: &str, name: &str, result: Result<()>) {
            match result {
                Ok(()) => {
                    log_info!(tag, "{name} PASSED");
                    self.passed += 1;
                }
                Err(e) => {
                    log_error!(tag, "{name} FAILED: {e}");
                    self.failed += 1;
                }
            }
        }
    }

    /// The board initialised by [`test_thread`], for the tests that drive real peripherals.
    pub(crate) fn hardware() -> &'static mut Hardware {
        match unsafe { &mut *&raw mut HARDWARE } {
            Some(hardware) => hardware,
            None => panic!("Hardware not initialized"),
        }
    }

    pub(super) fn test_thread(_thread: Box<dyn ThreadFn>, _: Option<ThreadParam>) -> Result<ThreadParam> {
        match osal_rs_tests::freertos::run_all_tests() {
            Ok(_) => osal_rs::log_info!(APP_TAG, "All tests passed!"),
            Err(e) => panic!("Tests failed with error: {:?}", e),
        };

        // Same handshake as the application main thread: the C side must
        // have finished the board setup before the drivers touch it.
        while unsafe { get_g_setup_called() } != 1 {}

        unsafe {
            HARDWARE = Some(Hardware::new());
        }

        if let Err(err) = hardware().init() {
            log_fatal!(APP_TAG, "Hardware error: {:?}", err);
            panic!("Hardware initialization failed");
        }

        let mut stats = TestStats::default();

        crate::drivers::run_all_tests(&mut stats);
        crate::apps::run_all_tests(&mut stats);

        log_info!(APP_TAG, "========================================");
        if stats.failed == 0 {
            log_info!(APP_TAG, "   FW tests PASSED: {} passed, 0 failed", stats.passed);
        } else {
            log_error!(APP_TAG, "   FW tests FAILED: {} passed, {} failed", stats.passed, stats.failed);
        }
        log_info!(APP_TAG, "   heap_free:{}", System::get_free_heap_size());
        log_info!(APP_TAG, "========================================");

        loop {
            System::delay(TickType::MAX);
        }
    }

}



