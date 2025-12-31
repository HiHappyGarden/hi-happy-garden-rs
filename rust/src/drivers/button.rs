use core::cell::RefCell;
use core::time::Duration;

use alloc::str;
use alloc::sync::Arc;

use osal_rs::{log_error, log_info, print};
use osal_rs::os::{Mutex, MutexFn, System, SystemFn, Thread, ThreadFn};
use osal_rs::utils::Result;

use crate::drivers::pico::{self, GpioPeripheral, GPIO_CONFIG_SIZE};
use crate::drivers::platform::Gpio;
use crate::traits::state::Initializable;

const APP_TAG: &str = "Button";

pub struct Button {
    gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>,
    thread: Thread,
}


impl Button {
    pub fn new(gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Self {
        Self {
            gpio,
            thread: Thread::new("button_trd", 1024, 3)
        }
    }
    
    pub fn init(&mut self, gpio: &mut Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Result<()> {
        log_info!(APP_TAG, "Init button");




        let gpio_clone = Arc::clone(gpio);

        self.thread.spawn_simple(move || {

            
            loop {
                match gpio_clone.lock().unwrap().read(&GpioPeripheral::Btn) {
                    Ok(value) => log_info!(APP_TAG, "Button:{}", value),
                    Err(_) => log_error!(APP_TAG, "Error reading button"),
                }

                System::delay_with_to_tick(Duration::from_millis(500u64));
            }

        })?;

        Ok(())
    }
}

