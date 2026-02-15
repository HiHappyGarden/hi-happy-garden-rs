/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/
 
use osal_rs::log_info;
use osal_rs::utils::Result;

use crate::assets::font_8x8::FONT_8X8;
use crate::assets::ic_wifi_off::IC_WIFI_OFF;
use crate::traits::button::{ButtonState, OnClickable};
use crate::traits::encoder::{EncoderDirection, OnRotatableAndClickable};
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::state::Initializable;

const APP_TAG: &str = "AppDisplay";

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

        self.lcd.invert_orientation()?;

        self.lcd.draw_pixel(1, 1, LCDWriteMode::ADD)?;
        self.lcd.draw_pixel(1, 2, LCDWriteMode::ADD)?;
        self.lcd.draw_pixel(1, 3, LCDWriteMode::ADD)?;
        self.lcd.draw_pixel(2, 4, LCDWriteMode::ADD)?;
        self.lcd.draw_pixel(2, 5, LCDWriteMode::ADD)?;
        

        self.lcd.draw_bitmap_image(30, 20, IC_WIFI_OFF.0, IC_WIFI_OFF.1, &IC_WIFI_OFF.2, LCDWriteMode::ADD)?;

        self.lcd.draw_str("ciao", 80, 50, &FONT_8X8)?;
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