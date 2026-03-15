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


use alloc::format;
#[allow(unused)]

use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::commons::{FIRST_ROW_Y, SECOND_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use Step::*;

#[derive(PartialEq, Eq)]
enum Step {
    Exit,
    Hour,
    Minute,
    Second,
    End
}

pub(super) struct Time<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    lcd: Arc<Mutex<T>>,
    hour: Option<u8>, // 0-23
    minute: Option<u8>, // 0-59
    second: Option<u8>, // 0-59
    step: Step,
    time: Option<DateTime>,
}

impl<T> Time<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{

    pub(super) fn new(lcd: Arc<Mutex<T>>) -> Self {
        Self {
            lcd,
            hour: None,
            minute: None,
            second: None,
            step: Hour,
            time: None,
        }
    }

    fn update_field(&mut self, signals: &mut EventBits) {
        match self.step {
            Exit => {},
            Hour => {
                if *signals & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
                    if let Some(hour) = self.hour {
                        self.hour = Some(if hour == 23 { 0 } else { hour + 1 });
                        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
                    }
                } else if *signals & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
                    if let Some(hour) = self.hour {
                        self.hour = Some(if hour == 0 { 23 } else { hour - 1 });
                        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
                    }
                }
            },
            Minute => {
                if *signals & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
                    if let Some(minute) = self.minute {
                        self.minute = Some(if minute == 59 { 0 } else { minute + 1 });
                        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
                    }
                } else if *signals & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
                    if let Some(minute) = self.minute {
                        self.minute = Some(if minute == 0 { 59 } else { minute - 1 });
                        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
                    }
                }
            },
            Second => {

                if *signals & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
                    if let Some(second) = self.second {
                        self.second = Some(if second == 59 { 0 } else { second + 1 });
                        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
                    }
                } else if *signals & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
                    if let Some(second) = self.second {
                        self.second = Some(if second == 0 { 59 } else { second - 1 });
                        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
                    }
                }
            },
            End => {},
        }

    }


    pub(super) fn draw(&mut self, signals: &mut EventBits, current_date_time: &DateTime, text: &impl AsSyncStr, date_time: Option<DateTime>, callback: Option<fn(Option<DateTime>)>) -> Result<()> {
        clean_context(&mut self.lcd)?;


        if self.time.is_none() {
            if let Some(dt) = date_time {
                self.hour = Some(dt.hour);
                self.minute = Some(dt.minute);
                self.second = Some(dt.second);
                self.time = Some(dt);
            }
        }


        if self.hour.is_none() || self.minute.is_none() || self.second.is_none() {
            self.hour = Some(current_date_time.hour);
            self.minute = Some(current_date_time.minute);
            self.second = Some(0);
            *signals |= DisplayFlag::Draw as u32;
        }


        if *signals & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            match self.step {
                Exit => self.step = Hour,
                Hour => self.step = Minute,
                Minute => self.step = Second,
                Second => self.step = End,
                End => {
                    self.time = DateTime::new_time( 
                        self.hour.unwrap_or(current_date_time.hour),
                        self.minute.unwrap_or(current_date_time.minute),
                        self.second.unwrap_or(current_date_time.second)
                            ).ok();
                    if let Some(cb) = callback {
                        cb(self.time);
                    }
                },
            }
            *signals |= DisplayFlag::Draw as u32;
        }

        if *signals & DisplayFlag::ButtonReleased as u32 != 0 {
            match self.step {
                Exit => {},
                Hour => self.step = Exit,
                Minute => self.step = Hour,
                Second => self.step = Minute,
                End => self.step = Second,
            }
            *signals |= DisplayFlag::Draw as u32;
        } 

        self.update_field(signals);

        if *signals & DisplayFlag::Draw as u32 == 0 {
            return Ok(())
        }

        let mut lcd = self.lcd.lock()?;

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(text.as_str(), current_date_time, (width - visible_width) / 2, visible_width, FONT_8X8[0], 100);

        lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;


        
        let date_str = format!(
            "{:02}:{:02}:{:02}",
            self.hour.unwrap_or(current_date_time.hour),
            self.minute.unwrap_or(current_date_time.minute),
            self.second.unwrap_or(current_date_time.second)
        );

        let date_width = date_str.chars().count() as u8 * FONT_8X8[0];
        let x_position = (width - date_width) / 2;

        lcd.draw_str(&date_str, x_position, SECOND_ROW_Y, &FONT_8X8)?;


        let (field_offset, field_width): (u8, u8) = match self.step {
            Hour  => (0, 16),
            Minute => (24, 16),
            Second   => (48, 16),
            _     => (0, 0),
        };
        if field_offset > 0 || field_width > 0 {
            lcd.draw_rect(x_position + field_offset, SECOND_ROW_Y + FONT_8X8[1], field_width, 2, LCDWriteMode::ADD)?;
        }

        if *signals & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            if self.step == End {
                    self.time = DateTime::new_time( 
                        self.hour.unwrap_or(current_date_time.hour),
                        self.minute.unwrap_or(current_date_time.minute),
                        self.second.unwrap_or(current_date_time.second)
                            ).ok();
                    if let Some(cb) = callback {
                        cb(self.time);
                    }
            }
        } 

        if *signals & DisplayFlag::ButtonReleased as u32 != 0 {
            if self.step == Exit {
                if let Some(cb) = callback {
                    cb(self.time);
                }
            }
        }
        
        Ok(())
    }

    #[allow(unused)]
    #[inline]
    pub fn get_date(&self) -> Option<DateTime> {
        self.time
    }
}

