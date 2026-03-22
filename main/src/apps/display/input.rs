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

use alloc::sync::Arc;
use osal_rs::os::{Mutex, MutexFn, System, SystemFn};
use osal_rs::os::types::EventBits;
use osal_rs::utils::{AsSyncStr, Bytes, Result};

use crate::apps::display::commons::{DisplayCallback, FIRST_ROW_Y, SECOND_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::LCDDisplayFn;

const LONG_PRESS_MS: u32 = 500;
pub const MAX_SIZE: usize = 64;

pub(super) struct Input<T>
where
    T: LCDDisplayFn + Sync + Send + Clone + 'static,
{
    lcd: Arc<Mutex<T>>,
    input: Option<Bytes<MAX_SIZE>>,
    original_input: Option<Bytes<MAX_SIZE>>,
    idx: usize,
    button_pressed_tick: u32,
    encoder_button_pressed_tick: u32,
}

impl<T> Input<T>
where
    T: LCDDisplayFn + Sync + Send + Clone + 'static,
{
    pub(super) fn new(lcd: Arc<Mutex<T>>) -> Self {
        Self { 
            lcd, 
            input: None,
            original_input: None,
            idx: 0,
            button_pressed_tick: 0,
            encoder_button_pressed_tick: 0,
        }
    }

    fn update_input(&mut self, signals: &mut EventBits) {
        if *signals & DisplayFlag::ButtonPressed as u32 != 0 {
            self.button_pressed_tick = System::get_tick_count();
            *signals &= !(DisplayFlag::ButtonPressed as u32);
        }
        if *signals & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
            if let Some(current) = self.input.as_ref() {
                let next_char = if current[self.idx] >= 0xFF {
                    b' ' // wrap from 255 back to 32 (space)
                } else {
                    current[self.idx] + 1
                };
                self.input.as_mut().unwrap()[self.idx] = next_char;
            } else {
                self.input = Some(Bytes::from_str("a"));
            }
            *signals |= DisplayFlag::Draw as u32;
        } else if *signals & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
            if let Some(current) = self.input.as_ref() {
                let prev_char = if current[self.idx] <= b' ' {
                    0xFF // wrap from 32 (space) back to 255
                } else {
                    current[self.idx] - 1
                };
                self.input.as_mut().unwrap()[self.idx] = prev_char;
            } else {
                self.input = Some(Bytes::from_str("a"));
            }
            *signals |= DisplayFlag::Draw as u32;
        }
         
        if *signals & DisplayFlag::EncoderButtonPressed as u32 != 0 {
            self.encoder_button_pressed_tick = System::get_tick_count();
            if let Some(mut input) = self.input {
                if self.idx < input.size() - 1 {
                    self.idx += 1;
                    let _ = input.push_char('a');
                    self.input = Some(input);
                }
            }
            *signals |= DisplayFlag::Draw as u32;
        } else if *signals & DisplayFlag::ButtonReleased as u32 != 0 {
            let elapsed = System::get_tick_count().wrapping_sub(self.button_pressed_tick);
            if elapsed >= LONG_PRESS_MS {
                // Long press: skip backspace, leave ButtonReleased set for draw() to handle
            } else {
                // Short press: backspace behaviour
                if let Some(mut input) = self.input {
                    if !input.is_empty() {
                        if self.idx > 0 {
                            self.idx -= 1;
                        }
                        let _ = input.pop();
                        *signals &= !(DisplayFlag::ButtonReleased as u32);
                    }
                    self.input = Some(input);
                }
            }
            *signals |= DisplayFlag::Draw as u32;
        }


    }

    pub(super) fn draw(
        &mut self,
        signals: &mut EventBits,
        date_time: &DateTime,
        text: &impl AsSyncStr,
        input: &dyn AsSyncStr,
        callback: DisplayCallback<Bytes<MAX_SIZE>>,
    ) -> Result<()> {
        clean_context(&mut self.lcd)?;

        if self.input.is_none() {
            let input_str = input.as_str();
            self.input = Some(Bytes::from_str(input_str));
            self.original_input = Some(Bytes::from_str(input_str));
            self.idx = input_str.len() - 1;
        } 

        self.update_input(signals);

        let mut lcd = self.lcd.lock()?;

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(text.as_str(), date_time, (width - visible_width) / 2, visible_width, FONT_8X8[0], 100);

        lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;

        if let Some(input) = &self.input {
            let raw = input.as_raw_bytes();
            if !raw.is_empty() {
                if raw.len() >= 16 {
                    // Input overflows display: show '<' + last 15 bytes starting at x=0
                    let offset = raw.len() - 15;
                    let mut display_buf = [0u8; 16];
                    display_buf[0] = b'<';
                    let src = &raw[offset..];
                    let copy_len = src.len().min(15);
                    display_buf[1..1 + copy_len].copy_from_slice(&src[..copy_len]);
                    lcd.draw_bytes(&display_buf[..1 + copy_len], 0, SECOND_ROW_Y, &FONT_8X8)?;
                } else {
                    lcd.draw_bytes(raw, 3, SECOND_ROW_Y, &FONT_8X8)?;
                }
            }
        }

        if *signals & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            let elapsed = System::get_tick_count().wrapping_sub(self.encoder_button_pressed_tick);
            if elapsed >= LONG_PRESS_MS {
                // Long press on encoder button: call callback with current input
                if let Some(input) = self.input {
                    if let Some(c) = callback {
                        c(Some(input.clone()), true);
                    }
                    self.input = Some(input);
                }
            }
            *signals &= !(DisplayFlag::EncoderButtonReleased as u32);
            *signals |= DisplayFlag::Draw as u32;
        } else if *signals & DisplayFlag::EncoderButtonPressed as u32 != 0 {
            if let Some(input) = self.input {
                if self.idx == input.size() - 1 {
                    if let Some(c) = callback {
                        c(Some(input.clone()), true);
                    }
                }
                self.input = Some(input);
            }
            *signals |= DisplayFlag::Draw as u32;
        } else if *signals & DisplayFlag::ButtonReleased as u32 != 0 {
            if let Some(input) = self.input {
                if input.is_empty() {
                    // Short press on empty input: cancel
                    if let Some(c) = callback {
                        c(None, false);
                    }
                } else {
                    // Long press: call back with the original unmodified text
                    if let Some(c) = callback {
                        c(self.original_input.clone(), false);
                    }
                }
                self.input = Some(input);
            }
            *signals |= DisplayFlag::Draw as u32;
        }



        Ok(())
    }

    #[allow(unused)]
    #[inline]
    pub(super) fn get_input(&self) -> Option<Bytes<MAX_SIZE>> {
        self.input.clone()
    }
}