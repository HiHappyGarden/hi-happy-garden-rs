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

#![allow(unused)]

use alloc::format;
use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::Mutex;
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::date_time_editor::{FieldEditor, FieldEditorConfig};
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::screen::{Screen, ScreenCallback, ScreenParam};

pub(super) struct Time(FieldEditor);

impl Screen for Time
{
    fn draw(&mut self, 
        lcd: &mut impl LCDDisplayFn,
        signals: &mut EventBits, 
        date_time: &DateTime, 
        text: &impl AsSyncStr, 
        param: ScreenParam, 
        callback: ScreenCallback
    ) -> Result<()> {

        self.0.draw(lcd, signals, date_time, text, param, callback)?;

        Ok(())
    }
}
  

impl Time {
    pub(super) const fn new() -> Self {
        Self(FieldEditor::new(FieldEditorConfig {
            field_min:    [0, 0, 0],
            field_max_fn: [
                |_, _, _| 23,
                |_, _, _| 59,
                |_, _, _| 59,
            ],
            field_wrap:   [true, true, true],
            formatter:    |h, m, s| format!("{:02}:{:02}:{:02}", h, m, s),
            underlines:   [(0, 16), (24, 16), (48, 16)],
            extractor:    |dt| (dt.hour as i32, dt.minute as i32, dt.second as i32),
            builder:      |h, m, s| DateTime::new_time(h as u8, m as u8, s as u8),
        }))
    }

    #[allow(unused)]
    #[inline]
    pub(super) fn get_time(&self) -> Option<DateTime> {
        self.0.get_result()
    }
}

