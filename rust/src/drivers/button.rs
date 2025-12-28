use core::cell::RefCell;
use core::time::Duration;

use alloc::str;
use alloc::sync::Arc;

use osal_rs::{log_error, log_info, print};
use osal_rs::os::{Mutex, MutexFn, System, SystemFn, Thread, ThreadFn};
use osal_rs::utils::Result;

use crate::drivers::pico::{self, GpioType};
use crate::drivers::platform::Gpio;
use crate::traits::button::Button as ButtonFn;
use crate::traits::gpio::Gpio as GpioFn;
use crate::traits::state::Initializable;

const APP_TAG: &str = "Button";

pub struct Button {
    gpio: Arc<Mutex<Gpio>>,
    thread: Thread,
}


impl ButtonFn for Button {
    fn new(gpio: Arc<Mutex<Gpio>>) -> Self {
        Self {
            gpio,
            thread: Thread::new("button_trd", 1024, 3)
        }
    }
    
    fn init(&mut self, gpio: &mut Arc<Mutex<Gpio>>) -> Result<()> {
        log_info!(APP_TAG, "Init button");




        let gpio_clone = Arc::clone(gpio);

        self.thread.spawn_simple(move || {

            loop {
                match gpio_clone.lock().unwrap().read(&GpioType::Btn) {
                    Ok(value) => log_info!(APP_TAG, "Button:{}", value),
                    Err(_) => log_error!(APP_TAG, "Error reading button"),
                }

                System::delay_with_to_tick(Duration::from_millis(500u64));
            }

        })?;

        Ok(())
    }
}

