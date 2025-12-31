
use osal_rs::log_info;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::Result;

use alloc::rc::Rc;

use alloc::sync::Arc;
use core::cell::RefCell;

use crate::drivers::pico::{GPIO_FN, GPIO_CONFIG, GPIO_CONFIG_SIZE};
use crate::traits::state::Initializable;

use crate::drivers::platform::{Button, Encoder, Gpio};

const APP_TAG: &str = "Hardware";

pub struct Hardware {
    gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>,
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
        let gpio = Arc::new(Mutex::new(Gpio::<GPIO_CONFIG_SIZE>::new(&GPIO_FN, unsafe { &mut GPIO_CONFIG })));
        let gpio_clone = Arc::clone(&gpio);
        
        Self { 
            gpio,
            encoder: Encoder::new(Arc::clone(&gpio_clone)),
            button: Button::new(Arc::clone(&gpio_clone)),
        }
        
    }
}