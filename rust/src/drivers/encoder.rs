use core::cell::RefCell;

use osal_rs::log_info;
use osal_rs::os::{Thread, ThreadFn};
use osal_rs::utils::Result;

use crate::drivers::platform::Gpio;
use crate::traits::state::Initializable;
use crate::traits::encoder::Encoder as EncoderFn;

const APP_TAG: &str = "Encoder";

pub struct Encoder<'a> {
    gpio: &'a RefCell<Gpio>,
}

impl<'a> EncoderFn<'a> for Encoder<'a> {
    fn new(gpio: &'a RefCell<Gpio>) -> Self {
        Self {
            gpio: gpio,
        }
    }

    fn init(&mut self, gpio: &'a mut RefCell<Gpio>) -> Result<()> {
        log_info!(APP_TAG, "Init encoder");

        Ok(())
    }
}
