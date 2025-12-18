
use osal_rs::os::{System, SystemFn, Thread, ThreadFn};
use alloc::boxed::Box;


#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_main() {





    #[cfg(feature = "tests")]
    {
        use osal_rs::println;



        println!("[hardware_main] Creating test_runner thread...");

        match Thread::new("test_runner", 4096, 3, Box::new(|_, _| {
            use osal_rs::utils::Error;


            match osal_rs_tests::freertos::run_all_tests() {
                Ok(_) => println!("[test_runner] All tests passed!"),
                Err(e) => panic!("Tests failed with error: {:?}", e)
            };

            Err(Error::Unhandled(""))
        })).spawn(None) {
            Ok(_spawned) =>  println!("[hardware_main] Thread spawned successfully!"),
            Err(e) => panic!("Failed to spawn test_runner thread: {:?}", e)
        };
        
    }
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_start_os() {
    System::start();
}
