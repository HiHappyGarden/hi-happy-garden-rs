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

#![allow(dead_code)]

use alloc::string::String;
use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::commons::{FIRST_ROW_Y, SECOND_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::screen::{ScreenCallback, ScreenParam};

#[derive(PartialEq, Eq)]
enum Step {
    Exit,
    Field1,
    Field2,
    Field3,
    End,
}

/// Configuration for a 3-field step-by-step editor.
/// Drives both the date editor (year / month / day) and the time editor
/// (hour / minute / second) from a single generic implementation.
pub(super) struct FieldEditorConfig {
    /// Minimum value for each field.
    pub field_min: [i32; 3],
    /// Returns the maximum value for field `i` given the current values of all
    /// three fields.  Signature: `fn(field1, field2, field3) -> max_i`.
    pub field_max_fn: [fn(i32, i32, i32) -> i32; 3],
    /// Whether each field wraps around at its min/max boundaries.
    pub field_wrap: [bool; 3],
    /// Formats the three field values into the display string.
    pub formatter: fn(i32, i32, i32) -> String,
    /// `(x_offset, width)` of the underline for Field1, Field2, Field3.
    pub underlines: [(u8, u8); 3],
    /// Extracts `(field1, field2, field3)` from a `DateTime`.
    pub extractor: fn(&DateTime) -> (i32, i32, i32),
    /// Builds a `DateTime` from the confirmed field values.
    pub builder: fn(i32, i32, i32) -> Result<DateTime>,
}

pub(super) struct FieldEditor<T>
where
    T: LCDDisplayFn + Sync + Send + Clone + 'static,
{
    lcd: Arc<Mutex<T>>,
    fields: [Option<i32>; 3],
    step: Step,
    result: Option<DateTime>,
    config: FieldEditorConfig,
}

impl<T> FieldEditor<T>
where
    T: LCDDisplayFn + Sync + Send + Clone + 'static,
{
    pub(super) fn new(lcd: Arc<Mutex<T>>, config: FieldEditorConfig) -> Self {
        Self {
            lcd,
            fields: [None, None, None],
            step: Step::Field1,
            result: None,
            config,
        }
    }

    fn update_field(&mut self, signals: &mut EventBits) {
        let idx = match self.step {
            Step::Field1 => 0,
            Step::Field2 => 1,
            Step::Field3 => 2,
            _ => return,
        };

        let delta: i32 = if *signals & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
            1
        } else if *signals & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
            -1
        } else {
            return;
        };

        if let Some(val) = self.fields[idx] {
            let f = [
                self.fields[0].unwrap_or(0),
                self.fields[1].unwrap_or(0),
                self.fields[2].unwrap_or(0),
            ];
            let min = self.config.field_min[idx];
            let max = (self.config.field_max_fn[idx])(f[0], f[1], f[2]);
            self.fields[idx] = Some(if self.config.field_wrap[idx] {
                let v = val + delta;
                if v > max { min } else if v < min { max } else { v }
            } else {
                val + delta
            });
            *signals |= DisplayFlag::Draw as u32;
        }
    }

    pub(super) fn draw(
        &mut self,
        signals: &mut EventBits,
        current_date_time: &DateTime,
        text: &impl AsSyncStr,
        param: ScreenParam, 
        callback: ScreenCallback,
    ) -> Result<()> {
        clean_context(&mut self.lcd)?;

        if self.result.is_none() {
            if let Some(dt) = param.date_time {
                let (f1, f2, f3) = (self.config.extractor)(&dt);
                self.fields = [Some(f1), Some(f2), Some(f3)];
                self.result = Some(dt);
            }
        }

        if self.fields[0].is_none() || self.fields[1].is_none() || self.fields[2].is_none() {
            let (f1, f2, f3) = (self.config.extractor)(current_date_time);
            self.fields = [Some(f1), Some(f2), Some(f3)];
            *signals |= DisplayFlag::Draw as u32;
        }

        if *signals & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            self.step = match self.step {
                Step::Exit   => Step::Field1,
                Step::Field1 => Step::Field2,
                Step::Field2 => Step::Field3,
                Step::Field3 => Step::End,
                Step::End    => Step::End,
            };
            *signals |= DisplayFlag::Draw as u32;
        }

        if *signals & DisplayFlag::ButtonReleased as u32 != 0 {
            self.step = match self.step {
                Step::Exit   => Step::Exit,
                Step::Field1 => Step::Exit,
                Step::Field2 => Step::Field1,
                Step::Field3 => Step::Field2,
                Step::End    => Step::Field3,
            };
            *signals |= DisplayFlag::Draw as u32;
        }

        self.update_field(signals);

        if *signals & DisplayFlag::Draw as u32 == 0 {
            return Ok(());
        }

        let mut lcd = self.lcd.lock()?;

        let (width, _) = lcd.get_size();
        let (visible_width, _) = lcd.get_visible_size();

        let (display_text, x_position) = scroll_text(
            text.as_str(),
            current_date_time,
            (width - visible_width) / 2,
            visible_width,
            FONT_8X8[0],
            100,
        );
        lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;

        let (c0, c1, c2) = (self.config.extractor)(current_date_time);
        let f = [
            self.fields[0].unwrap_or(c0),
            self.fields[1].unwrap_or(c1),
            self.fields[2].unwrap_or(c2),
        ];

        let value_str = (self.config.formatter)(f[0], f[1], f[2]);
        let str_width = value_str.chars().count() as u8 * FONT_8X8[0];
        let x_pos = (width - str_width) / 2;
        lcd.draw_str(&value_str, x_pos, SECOND_ROW_Y, &FONT_8X8)?;

        let (field_offset, field_width) = match self.step {
            Step::Field1 => self.config.underlines[0],
            Step::Field2 => self.config.underlines[1],
            Step::Field3 => self.config.underlines[2],
            _            => (0, 0),
        };
        if field_offset > 0 || field_width > 0 {
            lcd.draw_rect(
                x_pos + field_offset,
                SECOND_ROW_Y + FONT_8X8[1],
                field_width,
                2,
                LCDWriteMode::ADD,
            )?;
        }

        drop(lcd);

        if *signals & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            if self.step == Step::End {
                self.result = (self.config.builder)(f[0], f[1], f[2]).ok();
                if let Some(cb) = callback {
                    let mut p = ScreenParam::default();
                    p.date_time = self.result;
                    cb(Some(p), true);
                }
            }
        }

        if *signals & DisplayFlag::ButtonReleased as u32 != 0 {
            if self.step == Step::Exit {
                if let Some(cb) = callback {
                    let mut p = ScreenParam::default();
                    p.date_time = self.result;
                    cb(Some(p), false);
                }
            }
        }

        Ok(())
    }

    #[allow(unused)]
    #[inline]
    pub(super) fn get_result(&self) -> Option<DateTime> {
        self.result
    }
}
