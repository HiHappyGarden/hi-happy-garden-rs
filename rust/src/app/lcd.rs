use osal_rs::log_info;

use crate::traits::button::{ButtonState, OnClickable};
use crate::traits::encoder::{EncoderDirection, OnRotatableAndClickable};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Lcd";

pub struct Lcd;

impl Lcd {
    pub const fn new() -> Self{
        Self
    }
}

impl Initializable for Lcd {
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        
        

        // hardware.get_button().set_on_click(Box::new(|state| {
        //     match state {
        //         ButtonState::Pressed => log_info!(APP_TAG, "Button Pressed"),
        //         ButtonState::Released => log_info!(APP_TAG, "Button Released"),
        //         ButtonState::None => {}
        //     }
        // }));

        // hardware.get_encoder().set_on_click(Box::new(|state| {
        //     match state {
        //         ButtonState::Pressed => log_info!(APP_TAG, "Encoder Pressed"),
        //         ButtonState::Released => log_info!(APP_TAG, "Encoder Released"),
        //         ButtonState::None => {}
        //     }
        // }));

        // hardware.get_encoder().set_on_rotate(Box::new(|direction, _position| {
        //     match direction {
        //         EncoderDirection::Clockwise => log_info!(APP_TAG, "Encoder Clockwise"),
        //         EncoderDirection::CounterClockwise => log_info!(APP_TAG, "Encoder CounterClockwise"),
        //     }
        // }));


        Ok(())
    }
}

impl OnClickable for Lcd {
    fn on_click(&mut self, state: ButtonState) {
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