use core::cell::RefCell;

use alloc::sync::Arc;
use osal_rs::log_info;
use osal_rs::os::{Mutex, Thread, ThreadFn};
use osal_rs::utils::Result;

use crate::drivers::platform::Gpio;
use crate::drivers::pico::{self, GpioPeripheral, GPIO_CONFIG_SIZE};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Encoder";

pub struct Encoder {
    gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>,
}

impl Encoder {
    pub fn new(gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Self {
        Self {
            gpio: gpio,
        }
    }

    pub fn init(&mut self, _gpio: &mut Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Result<()> {
        log_info!(APP_TAG, "Init encoder");

        Ok(())
    }
}
