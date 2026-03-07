/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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

use alloc::string::String;
use alloc::sync::Arc;
use osal_rs::os::types::EventBits;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::{AsSyncStr, Result};

use crate::apps::display::commons::clean_context;
use crate::apps::signals::display::DisplayFlag;
use crate::assets::font_8x8::FONT_8X8;
use crate::assets::ic_check_off::IC_CHECK_OFF;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};

pub(super) struct Check<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    lcd: Arc<Mutex<T>>,
}

impl<T> Check<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{


    pub(super) fn new(lcd: Arc<Mutex<T>>) -> Self {
        Self {
            lcd,
        }
    }

    pub(super) fn draw(&mut self, signals: &mut EventBits, date_time: &DateTime, text: &impl AsSyncStr, _check: bool) -> Result<()> {
        clean_context(&mut self.lcd)?;
        
        let mut lcd = self.lcd.lock()?;

        let (width, _) = lcd.get_size(); // Get the display size (width, height)

        // LCD configuration: 132px total, first 2px and last 2px not visible
        // Visible area: 128px (from x=2 to x=130)
        // Font 8x8: each character is 8px wide
        // Maximum displayable characters: 16 (128px / 8px)
        const VISIBLE_WIDTH: u8 = 128;
        const CHAR_WIDTH: u8 = 8;
        const MAX_CHARS: usize = (VISIBLE_WIDTH / CHAR_WIDTH) as usize; // 16 characters
        const MARGIN_LEFT: u8 = 2; // Left margin (invisible pixels)
        
        let text_str = text.as_str();
        let text_len = text_str.chars().count();
        
        // Storage for scrolled text (if needed)
        let mut scrolled_text = String::new();
        
        let (display_text, x_position) = if text_len <= MAX_CHARS {
            // Text fits in display: center it
            let text_width = (text_len as u8) * CHAR_WIDTH;
            let x_pos = MARGIN_LEFT + (VISIBLE_WIDTH - text_width) / 2;
            (text_str, x_pos)
        } else {
            // Text is longer than display: implement scrolling
            // Update scroll position every ~100ms
            const SCROLL_DELAY_MS: u64 = 100;
            
            // Calculate scroll offset based on time
            // Add pause at start and end (2 extra positions)
            let scroll_positions = text_len - MAX_CHARS + 4; // +4 for pause at ends (2 at start, 2 at end)
            let scroll_index = ((date_time.millis as u64 / SCROLL_DELAY_MS) % (scroll_positions as u64)) as usize;
            
            // Calculate actual character offset (with pause handling)
            let char_offset = if scroll_index < 2 {
                0 // Pause at the beginning
            } else if scroll_index >= scroll_positions - 2 {
                text_len - MAX_CHARS // Pause at the end
            } else {
                scroll_index - 2 // Normal scrolling
            };
            
            // Extract the visible substring
            scrolled_text = text_str
                .chars()
                .skip(char_offset)
                .take(MAX_CHARS)
                .collect();
            
            (scrolled_text.as_str(), MARGIN_LEFT)
        };
        
        lcd.draw_str(display_text, x_position, 30, &FONT_8X8)?;

        lcd.draw_bitmap_image((width  / 2 ) - (IC_CHECK_OFF.0 / 2), 45, IC_CHECK_OFF.0, IC_CHECK_OFF.1, &IC_CHECK_OFF.2, LCDWriteMode::ADD)?;

        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
        Ok(())
    }
}

