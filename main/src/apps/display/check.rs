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

use crate::apps::display::commons::{FIRST_ROW_Y, SCROLL_DELAY_MS, SECOND_ROW_Y, clean_context, consume_event, has_event, request_draw, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::assets::ic_check_off::IC_CHECK_OFF;
use crate::assets::ic_check_on::IC_CHECK_ON;
use crate::assets::types::Icon;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenCallback, ScreenParam};

pub(in crate::apps) struct Check 
{
    icon: Icon<120>,
    checked: Option<bool>,
}

impl Screen<bool> for Check
{
    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        signal: &mut EventBits, 
        _: &Arc<Mutex<dyn RTC + 'static>>, 
        text: &dyn AsSyncStr, 
        param: ScreenParam, 
        callback: ScreenCallback
    ) -> Result<()> {

        clean_context(lcd)?;

        if self.checked.is_none() {
            if param.check.unwrap_or(false) { 
                self.icon = IC_CHECK_ON;
                self.checked = Some(true);
            } else {
                self.icon = IC_CHECK_OFF;
                self.checked = Some(false);
            }
        }

        self.update_icon(signal);

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(
            text.as_str(),
            signal,
            (width - visible_width) / 2, visible_width,
            FONT_8X8[0],
            SCROLL_DELAY_MS
        );

        lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;

        lcd.draw_bitmap_image((width  / 2 ) - (self.icon.0 / 2), SECOND_ROW_Y, self.icon.0, self.icon.1, &self.icon.2, LCDWriteMode::ADD)?;

        if consume_event(signal, DisplayFlag::EncoderButtonReleased) {
            if self.icon.2 == IC_CHECK_ON.2 {
                self.checked = Some(true);
                if let Some(ref cb) = callback {
                    let mut p = ScreenParam::default();
                    p.check = self.checked;
                    cb(Some(p), true);
                }
            } else {
                self.checked = Some(false);
                if let Some(ref cb) = callback {
                    let mut p = ScreenParam::default();
                    p.check = self.checked;
                    cb(Some(p), true);
                }
            };
            request_draw(signal); // Set the flag to indicate that the display should be redrawn 
        }

        if consume_event(signal, DisplayFlag::ButtonReleased) {
            if let Some(ref cb) = callback {
                cb(None, false);
            }
            request_draw(signal); // Set the flag to indicate that the display should be redrawn 
        }

        Ok(())
        
    }

    fn get_value(&self) -> Result<bool> {
        self.checked.ok_or(Error::NullPtr)
    }
}

impl Check {

    pub(in crate::apps) const fn new() -> Self {
        Self {
            icon: IC_CHECK_OFF,
            checked: None,
        }
    }

    fn update_icon(&mut self, signal: &mut EventBits) {
        if has_event(*signal, DisplayFlag::EncoderRotatedClockwise) || has_event(*signal, DisplayFlag::EncoderRotatedCounterClockwise) {
            self.icon = if self.icon.2 == IC_CHECK_OFF.2 {
                IC_CHECK_ON
            } else {
                IC_CHECK_OFF
            };
            request_draw(signal); // Set the flag to indicate that the display should be redrawn
        }
    }
}

