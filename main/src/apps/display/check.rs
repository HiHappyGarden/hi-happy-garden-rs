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

use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::commons::{clean_context, scroll_text};
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
}

impl<T> Check<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{

    pub(super) fn new(lcd: Arc<Mutex<T>>) -> Self {
        Self {
            lcd,
            icon: IC_CHECK_OFF,
        }
    }

    fn update_icon(&mut self, signals: &mut EventBits, check: Option<bool>) {
        if let Some(check) = check {
            if check {
                self.icon = IC_CHECK_ON; // Update with the appropriate icon based on the check state
            } else {
                self.icon = IC_CHECK_OFF; // Update with the appropriate icon based on the check state
            }
            *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that
        }
        else if *signals & DisplayFlag::EncoderRotatedClockwise as u32 != 0 || *signals & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {

            self.icon = if self.icon.2 == IC_CHECK_OFF.2 {
                IC_CHECK_ON
            } else {
                IC_CHECK_OFF
            };
            *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn
        }
    }

    pub(super) fn draw(&mut self, signals: &mut EventBits, date_time: &DateTime, text: &impl AsSyncStr, check: Option<bool>) -> Result<()> {
        clean_context(&mut self.lcd)?;

        self.update_icon(signals, check);

        let mut lcd = self.lcd.lock()?;

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(text.as_str(), date_time, (width - visible_width) / 2, visible_width, FONT_8X8[0], 100);

        lcd.draw_str(&display_text, x_position, 30, &FONT_8X8)?;

        lcd.draw_bitmap_image((width  / 2 ) - (self.icon.0 / 2), 45, self.icon.0, self.icon.1, &self.icon.2, LCDWriteMode::ADD)?;

        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
        Ok(())
    }
}

