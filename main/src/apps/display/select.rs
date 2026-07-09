/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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

use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::Mutex;
use osal_rs::utils::{AsSyncStr, Bytes, Error, Result};

use crate::apps::display::commons::{FIRST_ROW_Y, SCROLL_DELAY_MS, SECOND_ROW_Y, clean_context, scroll_text};
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenCallback, ScreenParam, ScreenSelections, screen_selections_new};

static NO_SELECTIONS: &str = "No selections available";

pub(in crate::apps) struct Select<const N: usize = 6> {
    index: u8,
    selections: Option<ScreenSelections<N>>,
}

impl<const N: usize> Screen<ScreenSelections<N>, u16, N> for Select<N> {
    fn draw(&mut self,
        lcd: &mut dyn LCDDisplayFn,
        signal: &mut EventBits,
        _: &Arc<Mutex<dyn RTC + 'static>>,
        text: &dyn AsSyncStr,
        param: ScreenParam<u16, N>,
        callback: ScreenCallback<u16, N>
    ) -> Result<()> {

        clean_context(lcd)?;

        if self.selections.is_none() {
            match &param.selects {
                Some(selections) => {
                    self.index = selections.iter().position(|(_, b)| *b).unwrap_or(0) as u8;
                    self.selections = Some(selections.clone());
                }
                None => {
                    self.index = 0;
                    self.selections = Some(screen_selections_new());
                }
            }
        }

        self.update_select(signal);

        let (width, _) = lcd.get_size(); 

        let (visible_width, _) = lcd.get_visible_size(); 

        let (display_text, x_position) = scroll_text(
            text.as_str(),
            signal,
            (width - visible_width) / 2,
            visible_width,
            FONT_8X8[0],
            SCROLL_DELAY_MS
        );

        lcd.draw_str(&display_text, x_position, FIRST_ROW_Y, &FONT_8X8)?;


        let text = if let Some(selections) = &self.selections {
            if let Some(selection) = selections.get(self.index as usize) {
                selection.0.as_str()
            } else {
                NO_SELECTIONS
            }
        } else {
            NO_SELECTIONS
        };

        let (display_text, x_position) = scroll_text(
            text,
            signal,
            (width - visible_width) / 2,
            visible_width,
            FONT_8X8[0],
            SCROLL_DELAY_MS
        );

        lcd.draw_str(&display_text, x_position, SECOND_ROW_Y, &FONT_8X8)?;
        if *signal & DisplayFlag::EncoderButtonReleased as u32 != 0 {
                if let Some(selected) = self.selections.as_ref() {
                    let mut p = ScreenParam::default();
                    p.selects = selected.clone().into();
                    if let Some(ref cb) = callback {
                        cb(Some(p), true);
                    }
                *signal |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
            } else {
                if let Some(ref cb) = callback {
                    cb(None, false);
                }
                *signal |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
            }
        }

        if *signal & DisplayFlag::ButtonReleased as u32 != 0 {
            if let Some(ref cb) = callback {
                cb(None, false);
            }
            *signal |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
        }

        
        Ok(())
        
    }

    #[inline]
    fn get_value(&self) -> Result<ScreenSelections<N>> {
        self.selections.clone().ok_or(Error::NullPtr)
    }

}

impl<const N: usize> Select<N> {

    pub(in crate::apps) const fn new() -> Self {
        Self {
            index: 0,
            selections: None,
        }
    }

    fn update_select(&mut self, signal: &mut EventBits) {

        let modulo = self.selections.as_ref().map_or(1, |s| s.len() as u8); // Get the length of selections or default to 1 to avoid division by zero

        if *signal & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
            self.index = self.index.wrapping_add(1) % modulo; // Increment index and wrap around using modulo
            *signal |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn
        } else  if *signal & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
            self.index = self.index.wrapping_sub(1) % modulo; // Decrement index and wrap around using modulo
            *signal |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn
        }
    }

}

