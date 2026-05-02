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

use osal_rs::os::{System, SystemFn};
use osal_rs::os::types::EventBits;
use osal_rs::utils::{AsSyncStr, Bytes, Result};

use super::commons::{FIRST_ROW_Y, SECOND_ROW_Y, LONG_PRESS_MS, MAX_SIZE, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::screen::{Screen, ScreenCallback, ScreenParam};

pub struct Input {
    input: Option<Bytes<MAX_SIZE>>,
    original_input: Option<Bytes<MAX_SIZE>>,
    idx: usize,
    button_pressed_tick: u32,
    encoder_button_pressed_tick: u32,
    secret_mode: bool, //todo: implement secret input mode (show '*' instead of actual chars) for password input, activated by long pressing the encoder button when on the first char of the input. In secret mode, the actual input is still stored in self.input and returned in the callback, but the display shows '*' for each char instead of the real chars. Long pressing the encoder button again would toggle back to normal mode.
}

impl Screen for Input
{
    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        signal: &mut EventBits, 
        date_time: &DateTime, 
        text: &dyn AsSyncStr, 
        param: ScreenParam, 
        callback: ScreenCallback
    ) -> Result<()> {

        clean_context(lcd)?;

        if self.input.is_none() {
            let input_str = param.input.unwrap_or_default();
            self.input = Some(Bytes::from_bytes(input_str.as_raw_bytes()));
            self.original_input = Some(Bytes::from_bytes(input_str.as_raw_bytes()));
            self.idx = input_str.len().saturating_sub(1);
        } 

        self.update_input(signal);

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(
            text.as_str(), 
            signal, 
            date_time, 
            (width - visible_width) / 2, visible_width,
            FONT_8X8[0],
            100
        );

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

        if *signal & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            let elapsed = System::get_tick_count().wrapping_sub(self.encoder_button_pressed_tick);
            if elapsed >= LONG_PRESS_MS {
                // Long press on encoder button: call callback with current input
                if let Some(input) = self.input {
                    if let Some(cb) = callback {
                        let mut p = ScreenParam::default();
                        p.input = Some(input.clone());
                        cb(Some(p), true);
                    }
                    self.input = Some(input);
                }
            }
            *signal &= !(DisplayFlag::EncoderButtonReleased as u32);
            *signal |= DisplayFlag::Draw as u32;
        } else if *signal & DisplayFlag::EncoderButtonPressed as u32 != 0 {
            if let Some(input) = self.input {
                if self.idx == input.size() - 1 {
                    if let Some(c) = callback {
                        let mut p = ScreenParam::default();
                        p.input = Some(input.clone());
                        c(Some(p), true);
                    }
                }
                self.input = Some(input);
            }
            *signal |= DisplayFlag::Draw as u32;
        } else if *signal & DisplayFlag::ButtonReleased as u32 != 0 {
            if let Some(input) = self.input {
                if input.is_empty() {
                    // Short press on empty input: cancel
                    if let Some(cb) = callback {
                        cb(None, false);
                    }
                } else {
                    // Long press: call back with the original unmodified text
                    if let Some(cb) = callback {
                        let mut p = ScreenParam::default();
                        p.input = self.original_input.clone();
                        cb(Some(p), false);
                    }
                }
                self.input = Some(input);
            }
            *signal |= DisplayFlag::Draw as u32;
        }

        Ok(())
    }
}


impl Input
{
    pub const fn new() -> Self {
        Self { 
            input: None,
            original_input: None,
            idx: 0,
            button_pressed_tick: 0,
            encoder_button_pressed_tick: 0,
            secret_mode: false,
        }
    }

    fn update_input(&mut self, signal: &mut EventBits) {
        if *signal & DisplayFlag::ButtonPressed as u32 != 0 {
            self.button_pressed_tick = System::get_tick_count();
            *signal &= !(DisplayFlag::ButtonPressed as u32);
        }
        if *signal & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
            if self.seed_input_if_empty() {
            } else if let Some(current) = self.input.as_ref() {
                let next_char = if current[self.idx] >= 0xFF {
                    b' ' // wrap from 255 back to 32 (space)
                } else {
                    current[self.idx] + 1
                };
                self.input.as_mut().unwrap()[self.idx] = next_char;
            }
            *signal |= DisplayFlag::Draw as u32;
        } else if *signal & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
            if self.seed_input_if_empty() {
            } else if let Some(current) = self.input.as_ref() {
                let prev_char = if current[self.idx] <= b' ' {
                    0xFF // wrap from 32 (space) back to 255
                } else {
                    current[self.idx] - 1
                };
                self.input.as_mut().unwrap()[self.idx] = prev_char;
            }
            *signal |= DisplayFlag::Draw as u32;
        }
         
        if *signal & DisplayFlag::EncoderButtonPressed as u32 != 0 {
            self.encoder_button_pressed_tick = System::get_tick_count();
            if self.seed_input_if_empty() {
            } else if let Some(mut input) = self.input {
                if self.idx < input.size() - 1 {
                    self.idx += 1;
                    let _ = input.push_char('a');
                }
                self.input = Some(input);
            }
            *signal |= DisplayFlag::Draw as u32;
        } else if *signal & DisplayFlag::ButtonReleased as u32 != 0 {
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
                        *signal &= !(DisplayFlag::ButtonReleased as u32);
                    }
                    self.input = Some(input);
                }
            }
            *signal |= DisplayFlag::Draw as u32;
        }
    }

    fn seed_input_if_empty(&mut self) -> bool {
        let needs_seed = match self.input.as_ref() {
            Some(input) => input.is_empty(),
            None => true,
        };

        if needs_seed {
            self.input = Some(Bytes::from_str("a"));
            self.idx = 0;
            true
        } else {
            false
        }
    }


    #[allow(unused)]
    #[inline]
    pub fn get_input(&self) -> Option<Bytes<MAX_SIZE>> {
        self.input.clone()
    }
}