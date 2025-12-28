use osal_rs::log_info;
use osal_rs::utils::Result;

use alloc::rc::Rc;
use core::cell::RefCell;

use crate::traits::state::Initializable;
use crate::traits::gpio::{Gpio as GpioFn};
use crate::traits::button::{Button as ButtonFn};
use crate::traits::encoder::{Encoder as EncoderFn};


use crate::drivers::platform::{Gpio, Button, Encoder};

const APP_TAG: &str = "Hardware";

pub struct Hardware<'a> {
    gpio: Rc<RefCell<Gpio>>,
    encoder: Encoder<'a>,
    button: Button<'a>,
}

impl<'a> Initializable<'a> for Hardware<'a> {
    fn init(&'a mut self) -> Result<()> {
        log_info!(APP_TAG, "Init hardware");

        self.gpio.borrow_mut().init()?;

        // Get raw pointer to RefCell<Gpio>
        let gpio_ref = Rc::as_ptr(&self.gpio);
        
        unsafe {
            // Cast to RefCell<Gpio> directly, not dyn trait
            self.encoder.init(&mut *(gpio_ref as *mut RefCell<Gpio>))?;
            self.button.init(&mut *(gpio_ref as *mut RefCell<Gpio>))?;
        }

        Ok(())
    } 
}

impl<'a> Hardware<'a> {
    pub fn new() -> Self {
        let gpio = Rc::new(RefCell::new(Gpio::new()));
        let gpio_ref = Rc::as_ptr(&gpio);
        
        unsafe {
            Self { 
                gpio,
                encoder: Encoder::new(&*(gpio_ref as *const RefCell<Gpio>)),
                button: Button::new(&*(gpio_ref as *const RefCell<Gpio>)),
            }
        }
    }
}