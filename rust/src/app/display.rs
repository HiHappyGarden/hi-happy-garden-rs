use osal_rs::log_info;
use osal_rs::utils::Result;

use crate::traits::button::{ButtonState, OnClickable};
use crate::traits::encoder::{EncoderDirection, OnRotatableAndClickable};
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Display";

pub struct Display<LC>
where LC: LCDDisplayFn
{
    lcd: LC
}

impl<LC> Display<LC> 
where LC: LCDDisplayFn
{
    pub const fn new(lcd: LC) -> Self{
        Self {
            lcd,
        }
    }

    pub fn draw(&mut self) -> Result<()> {

        

        self.lcd.draw_pixel(1, 1, LCDWriteMode::ADD)?;
        self.lcd.draw_pixel(1, 2, LCDWriteMode::ADD)?;
        self.lcd.draw_pixel(1, 3, LCDWriteMode::ADD)?;
        self.lcd.draw_pixel(2, 4, LCDWriteMode::ADD)?;
        self.lcd.draw_pixel(2, 5, LCDWriteMode::ADD)?;
        self.lcd.draw()?;

        Ok(())
    }
}

impl<LC> Initializable for Display<LC>
where LC: LCDDisplayFn
{
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        log_info!(APP_TAG, "Init LCD");

        Ok(())
    }
}

impl<LC> OnClickable for Display<LC>
where LC: LCDDisplayFn
{
    fn on_click(&self, state: ButtonState) {
        match state {
            ButtonState::Pressed => log_info!(APP_TAG, "Button Pressed"),
            ButtonState::Released => log_info!(APP_TAG, "Button Released"),
            ButtonState::None => {}
        }
    }
}

impl<LC> OnRotatableAndClickable for Display<LC>
where LC: LCDDisplayFn
{
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