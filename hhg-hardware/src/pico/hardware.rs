
use osal_rs::os::{System, SystemFn, Thread, ThreadFn};
use osal_rs::log::set_enable_color;
use alloc::boxed::Box;


#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_main() {
    set_enable_color(false);




    #[cfg(feature = "tests")]
    {
        const APP_TAG: &str = "hardware_main";
        use osal_rs::{log_info};

        log_info!(APP_TAG, "Creating test_runner thread...");

        match Thread::new("test_runner", 4096, 3, Box::new(|_, _| {
            use osal_rs::utils::Error;


            match osal_rs_tests::freertos::run_all_tests() {
                Ok(_) => log_info!(APP_TAG, "All tests passed!"),
                Err(e) => panic!("Tests failed with error: {:?}", e)
            };

            Err(Error::Unhandled(""))
        })).spawn(None) {
            Ok(_spawned) =>  log_info!(APP_TAG, "Thread spawned successfully!"),
            Err(e) => panic!("Failed to spawn test_runner thread: {:?}", e)
        };
        
    }
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_start_os() {
    System::start();
}
