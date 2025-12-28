use core::cell::RefCell;
use core::time::Duration;

use alloc::str;

use osal_rs::{log_error, log_info, print};
use osal_rs::os::{System, SystemFn, Thread, ThreadFn};
use osal_rs::utils::Result;

use crate::drivers::pico::{self, GpioType};
use crate::drivers::platform::Gpio;
use crate::traits::button::Button as ButtonFn;
use crate::traits::gpio::Gpio as GpioFn;
use crate::traits::state::Initializable;

const APP_TAG: &str = "Button";

pub struct Button<'a> {
    gpio: &'a RefCell<Gpio>,
    
}


impl<'a> ButtonFn<'a> for Button<'a> {
    fn new(gpio: &'a RefCell<Gpio>) -> Self {
        Self {
            gpio,
        }
    }
    
    fn init(&mut self, gpio: &'a mut RefCell<Gpio>) -> Result<()> {
        log_info!(APP_TAG, "Init button");

        // let _thread = match Thread::new("main_thread", 4096, 3, |_thread, _param| {

        //     log_info!(APP_TAG, "Button main thread started");

        //     loop {
        //         match gpio.borrow().read(&GpioType::Btn) {
        //             Ok(value) => log_info!(APP_TAG, "Button:{}", value),
        //             Err(_) => log_error!(APP_TAG, "Error reading button"),
        //         }

        //         System::delay_with_to_tick(Duration::from_millis(500u64));
        //     }


        // }).spawn(gpio.borrow().clone()) {
        //     Ok(spawned) =>  {
        //         log_info!(APP_TAG, "Start main thread\r\n");
        //         spawned
        //     }
        //     Err(e) => panic!("Failed to spawn main_thread: {:?}", e)
        // };

        Ok(())
    }
}

