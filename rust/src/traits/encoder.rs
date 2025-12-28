use core::cell::RefCell;

use osal_rs::utils::Result;

use crate::drivers::platform::Gpio;

pub trait Encoder<'a> {
    fn new(gpio: &'a RefCell<Gpio>) -> Self
    where 
        Self: Sized;

    fn init(&mut self, gpio: &'a mut RefCell<Gpio>) -> Result<()>;
}