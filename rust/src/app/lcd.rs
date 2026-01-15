use osal_rs::log_info;
use osal_rs::utils::Result;

use crate::traits::button::{ButtonState, OnClickable};
use crate::traits::encoder::{EncoderDirection, OnRotatableAndClickable};
use crate::traits::lcd_display::{LCDDisplay, LCDWriteMode};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Lcd";

pub struct Lcd {

}

impl Lcd {
    pub const fn new() -> Self{
        Self {

        }
    }

    pub fn draw(&mut self, display: &mut impl LCDDisplay) -> Result<()> {


        display.draw_pixel(1, 1, LCDWriteMode::ADD)?;
        display.draw_pixel(1, 2, LCDWriteMode::ADD)?;
        display.draw_pixel(1, 3, LCDWriteMode::ADD)?;
        display.draw_pixel(2, 4, LCDWriteMode::ADD)?;
        display.draw_pixel(2, 5, LCDWriteMode::ADD)?;

        display.draw()?;

        Ok(())
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
    fn on_rotable(&self, direction: EncoderDirection, position: i32) {
        match direction {
            EncoderDirection::Clockwise => log_info!(APP_TAG, "Encoder Clockwise pos:{position}"),
            EncoderDirection::CounterClockwise => log_info!(APP_TAG, "Encoder CounterClockwise pos:{position}"),
        }
    }

    fn on_click(&self, state: ButtonState) {
        match state {
            ButtonState::Pressed => log_info!(APP_TAG, "Encoder Pressed"),
            ButtonState::Released => log_info!(APP_TAG, "Encoder Released"),
            ButtonState::None => {}
        }
    }
}