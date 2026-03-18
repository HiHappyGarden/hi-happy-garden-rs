/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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
#![allow(unused)]

use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::commons::{FIRST_ROW_Y, SECOND_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::assets::ic_check_off::IC_CHECK_OFF;
use crate::assets::ic_check_on::IC_CHECK_ON;
use crate::assets::types::Icon;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};

pub(super) struct Check<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    lcd: Arc<Mutex<T>>,
    icon: Icon<120>,
    checked: Option<bool>,
}

impl<T> Check<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{

    pub(super) fn new(lcd: Arc<Mutex<T>>) -> Self {
        Self {
            lcd,
            icon: IC_CHECK_OFF,
            checked: None,
        }
    }

    fn update_icon(&mut self, signals: &mut EventBits) {
        if *signals & DisplayFlag::EncoderRotatedClockwise as u32 != 0 || *signals & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
            self.icon = if self.icon.2 == IC_CHECK_OFF.2 {
                IC_CHECK_ON
            } else {
                IC_CHECK_OFF
            };
            *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn
        }
    }

    pub(super) fn draw(&mut self, 
        signals: &mut EventBits, 
        date_time: &DateTime, 
        text: &impl AsSyncStr, 
        check: bool, 
        callback: Option<fn(Option<bool>)>
    ) -> Result<()> {
        clean_context(&mut self.lcd)?;

        if self.checked.is_none() {
            if check { 
                self.icon = IC_CHECK_ON;
                self.checked = Some(true);
            } else {
                self.icon = IC_CHECK_OFF;
                self.checked = Some(false);
            }
        }

        self.update_icon(signals);

        let mut lcd = self.lcd.lock()?;

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(text.as_str(), date_time, (width - visible_width) / 2, visible_width, FONT_8X8[0], 100);

        lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;

        lcd.draw_bitmap_image((width  / 2 ) - (self.icon.0 / 2), SECOND_ROW_Y, self.icon.0, self.icon.1, &self.icon.2, LCDWriteMode::ADD)?;

        if *signals & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            if self.icon.2 == IC_CHECK_ON.2 {
                self.checked = Some(true);
                if let Some(cb) = callback {
                    cb(Some(true));
                }
            } else {
                self.checked = Some(true);
                if let Some(cb) = callback {
                    cb(Some(false));
                }
            };
        }

        if *signals & DisplayFlag::ButtonReleased as u32 != 0 {
            if let Some(cb) = callback {
                cb(None);
            }
        }

        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
        Ok(())
    }

    #[allow(unused)]
    #[inline]
    pub(super) fn is_checked(&self) -> bool {
        self.checked.unwrap_or(false)
    }
}

