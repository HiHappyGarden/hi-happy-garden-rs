use osal_rs::log_info;

use crate::traits::button::{ButtonState, OnClickable};
use crate::traits::encoder::{EncoderDirection, OnRotatableAndClickable};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Lcd";

#[derive(Clone)]
pub struct Lcd;

impl Lcd {
    pub const fn new() -> Self{
        Self
    }
}

impl Initializable for Lcd {
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        log_info!(APP_TAG, "Init LCD");

        Ok(())
    }
}

impl OnClickable for Lcd {
    fn on_click(&self, state: ButtonState) {
        match state {
            ButtonState::Pressed => log_info!(APP_TAG, "Button Pressed"),
            ButtonState::Released => log_info!(APP_TAG, "Button Released"),
            ButtonState::None => {}
        }
    }
}

impl OnRotatableAndClickable for Lcd {
    fn on_rotable(&mut self, direction: EncoderDirection, position: i32) {
        match direction {
            EncoderDirection::Clockwise => log_info!(APP_TAG, "Encoder Clockwise pos:{position}"),
            EncoderDirection::CounterClockwise => log_info!(APP_TAG, "Encoder CounterClockwise pos:{position}"),
        }
    }

    fn on_click(&mut self, state: ButtonState) {
        match state {
            ButtonState::Pressed => log_info!(APP_TAG, "Encoder Pressed"),
            ButtonState::Released => log_info!(APP_TAG, "Encoder Released"),
            ButtonState::None => {}
        }
    }
}