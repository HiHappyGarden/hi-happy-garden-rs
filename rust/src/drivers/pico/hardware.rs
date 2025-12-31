
use osal_rs::log_info;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::Result;

use alloc::rc::Rc;

use alloc::sync::Arc;
use core::cell::RefCell;

use crate::traits::state::Initializable;


use crate::drivers::platform::{Gpio, Button, Encoder};

const APP_TAG: &str = "Hardware";

pub struct Hardware {
    gpio: Arc<Mutex<Gpio>>,
    encoder: Encoder,
    button: Button,
}

impl Initializable for Hardware {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init hardware");

        self.gpio.lock()?.init()?;

        self.encoder.init(&mut Arc::clone(&self.gpio))?;

        self.button.init(&mut Arc::clone(&self.gpio))?;

        Ok(())
    } 
}

impl Hardware {
    pub fn new() -> Self {
        let gpio = Arc::new(Mutex::new(Gpio::new()));
        let gpio_clone = Arc::clone(&gpio);
        
        Self { 
            gpio,
            encoder: Encoder::new(Arc::clone(&gpio_clone)),
            button: Button::new(Arc::clone(&gpio_clone)),
        }
        
    }
}