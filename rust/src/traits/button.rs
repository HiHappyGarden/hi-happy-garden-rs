
use alloc::sync::Arc;
use osal_rs::os::Mutex;
use osal_rs::utils::Result;

use crate::drivers::platform::Gpio;


pub trait Button {
    fn new(gpio: Arc<Mutex<Gpio>>) -> Self
    where 
        Self: Sized;

    fn init(&mut self, gpio: &mut Arc<Mutex<Gpio>>) -> Result<()>;
}