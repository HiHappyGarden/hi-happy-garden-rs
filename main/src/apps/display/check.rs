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

use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::Mutex;
use osal_rs::utils::{AsSyncStr, Error, Result};

use crate::apps::display::commons::{FIRST_ROW_Y, SCROLL_DELAY_MS, SECOND_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::assets::ic_check_off::IC_CHECK_OFF;
use crate::assets::ic_check_on::IC_CHECK_ON;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::rtc::RTC;
use crate::traits::screen::{Answer, Screen, ScreenParam};

#[derive(Clone, PartialEq, Eq)]
pub(in crate::apps) struct Check 
{
    value: bool,
    checked: Option<bool>,
}

impl Screen<bool> for Check
{
    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        signal: &mut EventBits, 
        _: &Arc<Mutex<dyn RTC + 'static>>, 
        text: &dyn AsSyncStr, 
        param: ScreenParam 
    ) -> Result<Answer> {

        clean_context(lcd)?;

        if self.checked.is_none() {
            match param {
                ScreenParam::Check(value) => {
                    self.value = value;
                    self.checked = Some(value);
                },
                _ => {
                    self.value = false;
                    self.checked = Some(false);
                }
            }
        }

        let redraw = self.update_icon(signal);

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(
            text.as_str(),
            signal,
            (width - visible_width) / 2, visible_width,
            FONT_8X8[0],
            SCROLL_DELAY_MS
        );

        lcd.draw_str(display_text.as_str(), x_position, FIRST_ROW_Y, &FONT_8X8)?;

        let icon = if self.value { IC_CHECK_ON } else { IC_CHECK_OFF };

        lcd.draw_bitmap_image((width  / 2 ) - (icon.0 / 2), SECOND_ROW_Y, icon.0, icon.1, &icon.2, LCDWriteMode::ADD)?;


        if redraw {
            *signal |= DisplayFlag::Draw as EventBits;
        }

        if *signal & DisplayFlag::EncoderButtonReleased as EventBits != 0 {
            self.checked = Some(self.value);
            return Ok(Answer::Confirmed(ScreenParam::Check(self.value)))
        }

        if *signal & DisplayFlag::ButtonReleased as EventBits != 0 {
            return Ok(Answer::Cancelled)
        }

        Ok(Answer::Pending)
        
    }

    fn get_value(&self) -> Result<bool> {
        self.checked.ok_or(Error::NullPtr)
    }


    fn clear(&mut self) {
        self.value = false;
        self.checked = None;
    }

}

impl Check {

    pub(in crate::apps) const fn new() -> Self {
        Self {
            value: false,
            checked: None,
        }
    }

    fn update_icon(&mut self, signal: &mut EventBits) -> bool {
        if *signal & DisplayFlag::EncoderRotatedClockwise as EventBits != 0 || *signal & DisplayFlag::EncoderRotatedCounterClockwise as EventBits != 0 {
            self.value = !self.value;
            true
        } else {
            false
        }
    }
}

