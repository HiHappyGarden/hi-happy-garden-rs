/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

#![allow(dead_code)]

use alloc::format;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::commons::{FIRST_ROW_Y, SECOND_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::integer::Integer;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::screen::{Screen, ScreenCallback, ScreenParam};

pub struct Number<N>
where
    N: Integer,
{
    number: Option<N>,
    min: N,
    max: N,
    result: Option<N>,
}

impl<N> Screen<N> for Number<N>
where
    N: Integer,
{
    fn draw(&mut self, 
        lcd: &mut impl LCDDisplayFn,
        signal: &mut EventBits, 
        date_time: &DateTime, 
        text: &impl AsSyncStr, 
        param: ScreenParam<N>, 
        callback: ScreenCallback<N>
    ) -> Result<()> {

        clean_context(lcd)?;


        if self.number.is_none() {
            self.number = param.number;
        } 

        self.update_number(signal);

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(text.as_str(), date_time, (width - visible_width) / 2, visible_width, FONT_8X8[0], 100);

        lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;

        let to_show = format!("{}", self.number.unwrap_or(self.min));

        let x_position = width - visible_width + (visible_width - (to_show.chars().count() as u8 * FONT_8X8[0])) / 2;

        lcd.draw_str(&to_show, x_position, SECOND_ROW_Y, &FONT_8X8)?;

        if *signal & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            self.result = self.number;
            if let Some(cb) = callback {
                let param = self.result.map(|n| { let mut p = ScreenParam::default(); p.number = Some(n); p });
                cb(param, true);
            }
        }

        if *signal & DisplayFlag::ButtonReleased as u32 != 0 {
            if let Some(cb) = callback {
                let param = self.number.map(|n| { let mut p = ScreenParam::default(); p.number = Some(n); p });
                cb(param, false);
            }
        }

        Ok(())
    }
}


impl<N> Number<N>
where
    N: Integer,
{
    pub const fn new(min: N, max: N) -> Self {
        Self { 
            number: None,
            min,
            max,
            result: None,
        }
    }

    fn update_number(&mut self, signal: &mut EventBits) {
        if *signal & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
            if let Some(current) = self.number {
                let new_value = current + N::one();
                self.number = Some(if new_value > self.max { self.min } else { new_value });
            } else {
                self.number = Some(self.min);
            }  
            *signal |= DisplayFlag::Draw as u32;
        } else if *signal & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
            if let Some(current) = self.number {
                let new_value = current - N::one();
                self.number = Some(if new_value < self.min { self.max } else { new_value });
            } else {
                self.number = Some(self.max);
            }
            *signal |= DisplayFlag::Draw as u32;
        } 
    }

    #[allow(unused)]
    #[inline]
    pub fn get_number(&self) -> Option<N> {
        self.result
    }
}