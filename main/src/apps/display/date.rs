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

#[allow(unused)]

use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::commons::{FIRST_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use Step::*;

enum Step {
    Year,
    Month,
    Day,
}

pub(super) struct Date<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    lcd: Arc<Mutex<T>>,
    year: Option<i32>, 
    month: Option<u8>, // 1-12
    mday: Option<u8>, // 1-31
    step: Step,
    date: Option<DateTime>,
}

impl<T> Date<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{

    pub(super) fn new(lcd: Arc<Mutex<T>>) -> Self {
        Self {
            lcd,
            year: None,
            month: None,
            mday: None,
            step: Year,
            date: None,
        }
    }



    pub(super) fn draw(&mut self, signals: &mut EventBits, current_date_time: &DateTime, text: &impl AsSyncStr, _date_time: Option<DateTime>, _callback: Option<fn(DateTime)>) -> Result<()> {
        clean_context(&mut self.lcd)?;

        if self.year.is_none() || self.month.is_none() || self.mday.is_none() {
            self.year = Some(current_date_time.year);
            self.month = Some(current_date_time.month);
            self.mday = Some(current_date_time.mday);
        }

        let mut lcd = self.lcd.lock()?;

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(text.as_str(), current_date_time, (width - visible_width) / 2, visible_width, FONT_8X8[0], 100);

        lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;


        if *signals & DisplayFlag::EncoderButtonReleased as u32 != 0 {

        }

        if *signals & DisplayFlag::ButtonReleased as u32 != 0 {

        } 

        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
        Ok(())
    }

    #[allow(unused)]
    #[inline]
    pub fn get_date(&self) -> Option<DateTime> {
        self.date
    }
}

