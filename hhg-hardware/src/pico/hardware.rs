use alloc::boxed::Box;
#[allow(unused_imports)]
use core::any::Any;

#[allow(unused_imports)]
use alloc::sync::Arc;
use osal_rs::os::{System, SystemFn, Thread, ThreadFn, ThreadParam};
use osal_rs::log::set_enable_color;
#[allow(unused_imports)]
use osal_rs::utils::Result;
use osal_rs::{log_info};

 #[cfg(not(feature = "tests"))]
static HARDWARE_THREAD: Option<Thread> = None;

const APP_TAG: &str = "hardware_main";


 #[cfg(not(feature = "tests"))]
fn hardware_main_thread(_thread: Box<dyn ThreadFn>, param: Option<ThreadParam>) -> Result<ThreadParam>{
    Ok(Arc::new(()))
}




#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_main() {
    set_enable_color(false);

    #[cfg(not(feature = "tests"))]
    {
        log_info!(APP_TAG, "Creating pico hardware test thread...");

        match Thread::new("hardware_main_thread", 4096, 3, hardware_main_thread).spawn(None) {
            Ok(_spawned) =>  log_info!(APP_TAG, "Thread spawned successfully!"),
            Err(e) => panic!("Failed to spawn hardware_main_thread: {:?}", e)
        };
    }



    #[cfg(feature = "tests")]
    {
        perform_tests();
    }
}







#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_start_os() {
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