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
use osal_rs::utils::{AsSyncStr, Bytes, Error, Result};

use super::commons::{FIRST_ROW_Y, SECOND_ROW_Y, MAX_SIZE, clean_context, scroll_text};
use crate::apps::display::commons::SCROLL_DELAY_MS;
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::screen::{Screen, ScreenCallback, ScreenParam};

const LONG_PRESS_TICK: u32 = 500;

pub struct Input {
    input: Option<Bytes<MAX_SIZE>>,
    original_input: Option<Bytes<MAX_SIZE>>,
    idx: usize,
    button_pressed_tick: u32,
    encoder_button_pressed_tick: u32,
    secret_mode: bool,
}

impl Screen<Bytes<MAX_SIZE>> for Input
{
    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        signal: &mut EventBits, 
        _: &DateTime, 
        text: &dyn AsSyncStr, 
        param: ScreenParam, 
        callback: ScreenCallback
    ) -> Result<()> {

        clean_context(lcd)?;

        if self.input.is_none() {
            let input_str = param.input.unwrap_or_default();
            self.input = Some(Bytes::from_bytes(input_str.as_raw_bytes()));
            self.original_input = Some(Bytes::from_bytes(input_str.as_raw_bytes()));
            if input_str.is_empty() {
                self.input = Some(Bytes::from_str("a"));
                self.idx = 0;
            } else {
                self.idx = input_str.len().saturating_sub(1);
            }
            if let Some(secret) = param.input_secret_mode {
                self.secret_mode = secret;
            }
        } 

        self.update_input(signal);

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

        //write the input text on the second row, with a '<' marker if the text is wider than the display and is being scrolled, and with a 3px left margin to avoid overlapping with the first column of the display which is not fully visible. The input text should be centered if it fits within the visible area, otherwise it should scroll circularly with a 4-space separator. If secret_mode is enabled, show '*' instead of the actual chars.
        if let Some(input) = &self.input {
            let raw = input.as_raw_bytes();
            if !raw.is_empty() {
                if raw.len() >= 16 {
                    // The input is wider than the 16-char line.
                    // Show an overflow marker ('<') plus the last 15 bytes.
                    let offset = raw.len() - 15;
                    let mut display_buf = [0u8; 16];
                    display_buf[0] = b'<';
                    let src = &raw[offset..];
                    let copy_len = src.len().min(15);
                    if self.secret_mode {
                        for i in 0..copy_len {
                            display_buf[1 + i] = b'*';
                        }
                        // Keep visible the character currently being edited.
                        if self.idx >= offset && self.idx < offset + copy_len {
                            let visible_idx = self.idx - offset;
                            display_buf[1 + visible_idx] = src[visible_idx];
                        }
                    } else {
                        display_buf[1..1 + copy_len].copy_from_slice(&src[..copy_len]);
                    }
                    lcd.draw_bytes(&display_buf[..1 + copy_len], 0, SECOND_ROW_Y, &FONT_8X8)?;
                } else if self.secret_mode {
                    let masked = [b'*'; 16];
                    let mut display_buf = masked;
                    if self.idx < raw.len() {
                        display_buf[self.idx] = raw[self.idx];
                    }
                    lcd.draw_bytes(&display_buf[..raw.len()], 3, SECOND_ROW_Y, &FONT_8X8)?;
                } else {
                    lcd.draw_bytes(raw, 3, SECOND_ROW_Y, &FONT_8X8)?;
                }
            }
        }

        //self.idx

        let mut x: u8 = 2;
        x = x.saturating_add((self.idx * 8).try_into().unwrap());

        lcd.draw_rect(x, SECOND_ROW_Y + 9, 8, 1, LCDWriteMode::INVERT)?;

        // Callback handling: encoder long press confirms the current input, while regular button long press restores the original input and short press cancels when the buffer becomes empty.
        if *signal & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            let elapsed = System::get_tick_count().wrapping_sub(self.encoder_button_pressed_tick);
            if elapsed >= LONG_PRESS_TICK {
                // Long press on encoder button: confirm the current input.
                if let Some(input) = self.input {
                    if let Some(cb) = callback {
                        let mut p = ScreenParam::default();
                        p.input = Some(input.clone());
                        cb(Some(p), true);
                    }
                    self.input = Some(input);
                }
            }
            self.encoder_button_pressed_tick = 0;
            *signal &= !(DisplayFlag::EncoderButtonReleased as u32);
            *signal |= DisplayFlag::Draw as u32;

        } else if *signal & DisplayFlag::ButtonReleased as u32 != 0 {
            let elapsed = System::get_tick_count().wrapping_sub(self.button_pressed_tick);
            if elapsed >= LONG_PRESS_TICK {
                if let Some(cb) = callback {
                    let mut p = ScreenParam::default();
                    p.input = self.original_input.clone();
                    cb(Some(p), false);
                }
            } else if self.input.as_ref().is_none_or(|input| input.is_empty()) {
                if let Some(cb) = callback {
                    cb(None, false);
                }
            }
            self.button_pressed_tick = 0;
            *signal &= !(DisplayFlag::ButtonReleased as u32);
            *signal |= DisplayFlag::Draw as u32;

        }

        Ok(())
    }

    fn get_value(&self) -> Result<Bytes<MAX_SIZE>> {
        self.input.clone().ok_or(Error::NullPtr)
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

        //encoder rotattion
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
            //pressing the encoder button doesn't immediately trigger an action, we wait for the release to determine if it was a short or long press, but we still need to record the tick count at the moment of the press to measure the duration later
            self.encoder_button_pressed_tick = System::get_tick_count();
            *signal &= !(DisplayFlag::EncoderButtonPressed as u32); 
        } else if *signal & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            let elapsed = System::get_tick_count().wrapping_sub(self.encoder_button_pressed_tick);
            if elapsed >= LONG_PRESS_TICK {
                // Long press: draw() will confirm the current input through the callback.
            } else {
                if self.seed_input_if_empty() {
                } else if let Some(mut input) = self.input {
                    let current_len = input.len();
                    if self.idx + 1 < current_len {
                        self.idx += 1;
                    } else if current_len < input.size() {
                        let _ = input.push_char('a');
                        self.idx = input.len().saturating_sub(1);
                    }
                    self.input = Some(input);
                }
                *signal |= DisplayFlag::Draw as u32;
            }
        } else if *signal & DisplayFlag::ButtonReleased as u32 != 0 {
            let elapsed = System::get_tick_count().wrapping_sub(self.button_pressed_tick);
            if elapsed >= LONG_PRESS_TICK {
                // Long press: draw() will restore the original input through the callback.
            } else {
                if let Some(mut input) = self.input {
                    if !input.is_empty() {
                        let _ = input.pop();
                        let new_len = input.len();
                        self.idx = new_len.saturating_sub(1);
                    }
                    self.input = Some(input);
                }
                *signal |= DisplayFlag::Draw as u32;
            }
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
}