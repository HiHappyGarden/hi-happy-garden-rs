/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
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

use osal_rs::os::types::EventBits;
use osal_rs::utils::{AsSyncStr, Error, Result};

use crate::apps::display::commons::{FIRST_ROW_Y, ONLY_ONE_ROW_Y, SCROLL_DELAY_MS, SECOND_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::screen::{Screen, ScreenCallback, ScreenParam};



pub struct Text;


impl Screen<()> for Text
{
    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        signal: &mut EventBits, 
        _: &DateTime, 
        text: &dyn AsSyncStr, 
        _: ScreenParam, 
        callback: ScreenCallback
    ) -> Result<()> {

        clean_context(lcd)?;
        
        let splitted_text = text.as_str().split("|");


        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 


        match splitted_text.clone().count() {
            0 => return Err(Error::Unhandled("Text cannot be empty")),
            1 => {
                let text = splitted_text.into_iter().next().unwrap_or_default();

                let (display_text, x_position) = scroll_text(text,
                    signal, 
                    (width - visible_width) / 2, visible_width,
                    FONT_8X8[0],
                    SCROLL_DELAY_MS
                );

                lcd.draw_str(&display_text, x_position, ONLY_ONE_ROW_Y, &FONT_8X8)?;
            },
            2 => {
                let mut iter = splitted_text.into_iter();
                let first_line = iter.next().unwrap_or_default();
                let second_line = iter.next().unwrap_or_default();

                let (display_text, x_position) = scroll_text(
                    first_line,
                    signal,
                    (width - visible_width) / 2, visible_width,
                    FONT_8X8[0],
                    SCROLL_DELAY_MS
                );


                lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;

                let (display_text, x_position) = scroll_text(
                    second_line,
                    signal,
                    (width - visible_width) / 2, visible_width,
                    FONT_8X8[0],
                    SCROLL_DELAY_MS
                );

                lcd.draw_str(&display_text, x_position, SECOND_ROW_Y, &FONT_8X8)?;
            },
            _ => return Err(Error::Unhandled("Text must contain at most one '|' character")),
            
        }

        if *signal & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            if let Some(cb) = callback {
                cb(Option::None, true);
            }
        }

        if *signal & DisplayFlag::ButtonReleased as u32 != 0 {
            if let Some(cb) = callback {
                cb(Option::None, false);
            }
        }

        Ok(())
    }

    fn get_value(&self) -> Result<()> {
        Ok(())
    }

}


impl Text {

    pub const fn new() -> Self {
        Self
    }

}