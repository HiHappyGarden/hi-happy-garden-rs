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

use alloc::string::String;
use alloc::sync::Arc;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::Result;

use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};


pub fn clean_context<T>(lcd: &mut Arc<Mutex<T>>) -> Result<()> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    let mut lcd = lcd.lock().unwrap();
    let (display_width, display_height) = lcd.get_size();
    let y_start = lcd.get_header_height();
    lcd.draw_rect(0, y_start, display_width, display_height - y_start, LCDWriteMode::REMOVE)
}

// LCD configuration: 132px total, first 2px and last 2px not visible
// Visible area: 128px (from x=2 to x=130)
// Font 8x8: each character is 8px wide
// Maximum displayable characters: 16 (128px / 8px)
/// Returns the text to display and the x position for the LCD.
/// If the text fits, it is centred; otherwise it scrolls circularly with 4-space separator.
pub fn scroll_text(text: &str, date_time: &DateTime, margin_left: u8, visible_width: u8, char_width: u8, scroll_delay_ms: u64) -> (String, u8) {
    let text_len = text.chars().count();

    let max_chars: usize = (visible_width / char_width) as usize; 

    if text_len <= max_chars {
        let text_width = (text_len as u8) * char_width;
        let x_pos = margin_left + (visible_width - text_width) / 2;
        (String::from(text), x_pos)
    } else {
        let total_millis = (date_time.second as u64 * 1000) + date_time.millis as u64;

        // Append 4 spaces as separator between end and beginning of the circular scroll
        let loop_text = ::alloc::format!("{}{}", text, "    ");
        let loop_len = loop_text.chars().count();

        let scroll_index = ((total_millis / scroll_delay_ms) % (loop_len as u64)) as usize;

        let scrolled: String = loop_text.chars().cycle().skip(scroll_index).take(max_chars).collect();
        (scrolled, margin_left)
    }
}
