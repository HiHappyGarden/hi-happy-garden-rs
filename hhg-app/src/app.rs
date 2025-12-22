use alloc::boxed::Box;
use alloc::sync::Arc;

use osal_rs::os::{Thread, ThreadFn, ThreadParam};

use osal_rs::utils::Result;
use osal_rs::{log_info};



static mut APP_THREAD: Option<Thread> = None;

const APP_TAG: &str = "app_main";


fn app_main_thread(_thread: Box<dyn ThreadFn>, _param: Option<ThreadParam>) -> Result<ThreadParam>{
    log_info!(APP_TAG, "Start app main");
    Ok(Arc::new(()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn app_main() {

    match Thread::new("app_main_thread", 4096, 3, app_main_thread).spawn(None) {
        Ok(spawned) => unsafe  { APP_THREAD = Some(spawned)},
        Err(e) => panic!("Failed to spawn app_main_thread: {:?}", e)
    };

}
