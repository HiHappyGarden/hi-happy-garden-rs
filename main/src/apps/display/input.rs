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
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::os::types::EventBits;
use osal_rs::utils::{AsSyncStr, Bytes, Result};

use crate::apps::display::commons::{FIRST_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::LCDDisplayFn;

pub struct Input<T>
where
    T: LCDDisplayFn + Sync + Send + Clone + 'static,
{
    lcd: Arc<Mutex<T>>,
    input: Option<Bytes<64>>,
    idx: usize,
    result: Option<Bytes<64>>,
}

impl<T> Input<T>
where
    T: LCDDisplayFn + Sync + Send + Clone + 'static,
{
    pub(super) fn new(lcd: Arc<Mutex<T>>) -> Self {
        Self { 
            lcd, 
            input: None,
            idx: 0,
            result: None 
        }
    }

    fn update_input(&mut self, signals: &mut EventBits) {
        if *signals & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
            if let Some(current) = self.input.as_ref().get(self.idx) {
                let next_char = if *current == b'z' {
                    b'a'
                } else if *current == b'Z' {
                    b'A'
                } else if *current == b'9' {
                    b'0'
                } else {
                    current + 1
                };
                self.input.as_mut().unwrap()[self.idx] = next_char;
            } else {
                self.input = Some(Bytes::from_str("a"));
            }
            *signals |= DisplayFlag::Draw as u32;
        } else if *signals & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
            if let Some(current) = self.input.as_ref().get(self.idx) {
                let prev_char = if *current == b'a' {
                    b'z'
                } else if *current == b'A' {
                    b'Z'
                } else if *current == b'0' {
                    b'9'
                } else {
                    current - 1
                };
                self.input.as_mut().unwrap()[self.idx] = prev_char;
            } else {
                self.input = Some(Bytes::from_str("a"));
            }
            *signals |= DisplayFlag::Draw as u32;
        }

        if *signals & DisplayFlag::EncoderButtonPressed as u32 != 0 {
            if let Some(input) = self.input.as_ref() {
                if self.idx < input.len() - 1 {
                    self.idx += 1;
                } else {
                    self.result = Some(input.clone());
                }
            }
            *signals |= DisplayFlag::Draw as u32;
        } else if *signals & DisplayFlag::ButtonReleased as u32 != 0 {
            if let Some(input) = self.input.as_ref() {
                if self.idx > 0 {
                    self.idx -= 1;
                    self.input.unwrap_or_default();
                } else {
                    self.result = Some(input.clone());
                }
            }
        }
         
    }

    pub(super) fn draw(
        &mut self,
        signals: &mut EventBits,
        date_time: &DateTime,
        text: &impl AsSyncStr,
        input: &dyn AsSyncStr,
        _callback: Option<fn(Option<Bytes<64>>)>,
    ) -> Result<()> {
        clean_context(&mut self.lcd)?;

        if self.input.is_none() {
            self.input = Some(Bytes::from_str(input.as_str()));
        } 

        self.update_input(signals);

        let mut lcd = self.lcd.lock()?;

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(text.as_str(), date_time, (width - visible_width) / 2, visible_width, FONT_8X8[0], 100);

        lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;



        Ok(())
    }

    #[allow(unused)]
    #[inline]
    pub(super) fn get_input(&self) -> Option<Bytes<64>> {
        self.input.clone()
    }
}