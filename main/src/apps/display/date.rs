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

use alloc::format;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::date_time_editor::{FieldEditor, FieldEditorConfig};
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::screen::{Screen, ScreenCallback, ScreenParam};

pub struct Date(FieldEditor);

impl Screen for Date
{
    fn draw(&mut self, 
        lcd: &mut impl LCDDisplayFn,
        signal: &mut EventBits, 
        date_time: &DateTime, 
        text: &impl AsSyncStr, 
        param: ScreenParam, 
        callback: ScreenCallback
    ) -> Result<()> {

        self.0.draw(lcd, signal, date_time, text, param, callback)?;

        Ok(())
    }
}
    
impl Date
{
    pub const fn new() -> Self {
        Self(FieldEditor::new(
            FieldEditorConfig {
            field_min:    [i32::MIN, 1, 1],
            field_max_fn: [
                |_, _, _| i32::MAX,
                |_, _, _| 12,
                |year, month, _| DateTime::days_in_month(month as u8, year) as i32,
            ],
            field_wrap:   [false, true, true],
            formatter:    |y, m, d| format!("{:04}-{:02}-{:02}", y, m, d),
            underlines:   [(0, 32), (40, 16), (64, 16)],
            extractor:    |dt| (dt.year, dt.month as i32, dt.mday as i32),
            builder:      |y, m, d| DateTime::new_date(y, m as u8, d as u8),
        }))
    }

    #[allow(unused)]
    #[inline]
    pub fn get_date(&self) -> Option<DateTime> {
        self.0.get_result()
    }
}

