
use osal_rs::os::{System, SystemFn};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_main() {

}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn hardware_start_os() {
    System::start();
}