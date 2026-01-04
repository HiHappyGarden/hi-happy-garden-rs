
use alloc::boxed::Box;
use alloc::sync::Arc;
use osal_rs::log_info;
use osal_rs::os::{Mutex, MutexFn};

use crate::drivers::platform::Hardware;
use crate::traits::button::{ButtonState, OnClickable};
use crate::traits::encoder::{EncoderDirection, OnRotatableAndClickable};
use crate::traits::hardware::{self, HardwareFn};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Lcd";

pub struct Lcd {
    hardware: Arc<Mutex<Hardware>>,
}

impl Lcd {
    pub const fn new(hardware: Arc<Mutex<Hardware>>,) -> Self{
        Self { hardware }
    }
}

impl Initializable for Lcd {
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        
        let mut hardware = self.hardware.lock()?;

        hardware.get_button().set_on_click(Box::new(|state| {
            match state {
                ButtonState::Pressed => log_info!(APP_TAG, "Button Pressed"),
                ButtonState::Released => log_info!(APP_TAG, "Button Released"),
                ButtonState::None => {}
            }
        }));

        hardware.get_encoder().set_on_click(Box::new(|state| {
            match state {
                ButtonState::Pressed => log_info!(APP_TAG, "Encoder Pressed"),
                ButtonState::Released => log_info!(APP_TAG, "Encoder Released"),
                ButtonState::None => {}
            }
        }));

        hardware.get_encoder().set_on_rotate(Box::new(|direction, _position| {
            match direction {
                EncoderDirection::Clockwise => log_info!(APP_TAG, "Encoder Clockwise"),
                EncoderDirection::CounterClockwise => log_info!(APP_TAG, "Encoder CounterClockwise"),
            }
        }));


        Ok(())
    }
}