use osal_rs::log_info;

use crate::traits::button::{ButtonState, OnClickable};
use crate::traits::encoder::{EncoderDirection, OnRotatableAndClickable};
use crate::traits::lcd_display::{LCDDisplay, LCDWriteMode};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Lcd";

pub struct Lcd<'a> {
    display: Option<&'a mut dyn LCDDisplay>,
}

impl<'a> Lcd<'a> {
    pub const fn new() -> Self{
        Self {
            display: None,
        }
    }

    pub fn set_display(&mut self, display: &'a mut dyn LCDDisplay) {


        //let _ = display.draw_rect(10, 0, 3, 3, LCDWriteMode::ADD);
        let _ = display.draw_pixel(10, 0, LCDWriteMode::ADD);

        let _ = display.draw();

        self.display = Some(display);
    }
}

impl Initializable for Lcd<'_> {
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        log_info!(APP_TAG, "Init LCD");

        Ok(())
    }
}

impl OnClickable for Lcd<'_> {
    fn on_click(&self, state: ButtonState) {
        match state {
            ButtonState::Pressed => log_info!(APP_TAG, "Button Pressed"),
            ButtonState::Released => log_info!(APP_TAG, "Button Released"),
            ButtonState::None => {}
        }
    }
}

impl OnRotatableAndClickable for Lcd<'_> {
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