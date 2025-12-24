use osal_rs::log_info;
use osal_rs::utils::Result;

use crate::traits::initializable::Initializable;
use crate::traits::gpio::{Gpio as CpioFn};
use super::gpio::*;

const APP_TAG: &str = "hardware";

pub struct Hardware {
    gpio: Gpio
}

impl Initializable for Hardware {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init hardware");

        log_info!(APP_TAG, "Init GPIO");
        self.gpio.init()?;

        Ok(())
    }

   
}

impl Hardware {
    pub fn new() -> Self {
        Self { 
            gpio: Gpio::new()
         }
    }
}