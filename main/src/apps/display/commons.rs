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
use core::sync::atomic::{AtomicU32, Ordering};

use alloc::string::String;
use osal_rs::os::{System, SystemFn};
use osal_rs::os::types::EventBits;
use osal_rs::utils::Result;

use crate::apps::signals::display::DisplayFlag;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};

#[allow(dead_code)]
pub(super) const ONLY_ONE_ROW_Y: u8 = 27;

pub(super) const FIRST_ROW_Y: u8 = 25;
pub(super) const SECOND_ROW_Y: u8 = 45;
pub(super) const MAX_SIZE: usize = 32;

static LAST_SCROLL_SLOT: AtomicU32 = AtomicU32::new(u32::MAX);
pub(super) const SCROLL_DELAY_MS: u64 = 200;

#[inline]
pub(super) fn has_event(signal: EventBits, flag: DisplayFlag) -> bool {
    signal & flag as u32 != 0
}

pub(super) fn consume_event(signal: &mut EventBits, flag: DisplayFlag) -> bool {
    if has_event(*signal, flag) {
        *signal &= !(flag as u32);
        true
    } else {
        false
    }
}

#[inline]
pub(super) fn request_draw(signal: &mut EventBits) {
    *signal |= DisplayFlag::Draw as u32;
}

macro_rules! get_datetime_from_rtc {

    ($rtc:expr, $error_flag:expr) => {{
        get_datetime_from_rtc!($rtc, $error_flag, true)
    }};
    ($rtc:expr, $error_flag:expr, $locale:expr) => {{
        use $crate::traits::signal::Signal;
        use osal_rs::os::MutexFn;

        $crate::drivers::date_time::DateTime::from_timestamp_locale($rtc.lock().unwrap().get_timestamp().unwrap_or(0), $locale)
            .unwrap_or_else(|_| {
                $crate::apps::signals::error::ErrorSignal::set($error_flag.into());
                $crate::drivers::date_time::DateTime::default()
            })
    }};
}
pub(in crate::apps) use get_datetime_from_rtc;

pub(super) fn clean_context(lcd: &mut dyn LCDDisplayFn) -> Result<()> 
{
    let (display_width, display_height) = lcd.get_size();
    let y_start = lcd.get_header_height();
    if y_start >= display_height {
        return Ok(());
    }
    lcd.draw_rect(0, y_start, display_width, display_height - y_start, LCDWriteMode::REMOVE)
}

// LCD configuration: 132px total, first 2px and last 2px not visible
// Visible area: 128px (from x=2 to x=130)
// Font 8x8: each character is 8px wide
// Maximum displayable characters: 16 (128px / 8px)
/// Returns the text to display and the x position for the LCD.
/// If the text fits, it is centred; otherwise it scrolls circularly with 4-space separator.
pub(super) fn scroll_text(
    text: &str, 
    signal: &mut EventBits, 
    margin_left: u8, 
    visible_width: u8, 
    char_width: u8, 
    scroll_delay_ms: u64
) -> (String, u8) {
    let text_len = text.chars().count();

    let max_chars: usize = (visible_width / char_width) as usize; 

    if text_len <= max_chars {
        let text_width = (text_len as u8) * char_width;
        let x_pos = margin_left + (visible_width - text_width) / 2;
        (String::from(text), x_pos)
    } else {
        // Use OS monotonic ticks to keep scroll cadence independent from RTC precision.
        let total_millis = System::get_tick_count() as u64;
        let scroll_slot = (total_millis / scroll_delay_ms) as u32;

        if LAST_SCROLL_SLOT.swap(scroll_slot, Ordering::Relaxed) != scroll_slot {
            *signal |= DisplayFlag::Draw as u32;
        }

        let loop_text = ::alloc::format!("{}{}", text, "  "); // Add spaces for separation
        let loop_len = loop_text.chars().count();

        let scroll_index = ((total_millis / scroll_delay_ms) % (loop_len as u64)) as usize;

        let scrolled: String = loop_text.chars().cycle().skip(scroll_index).take(max_chars).collect();
        (scrolled, margin_left)
    }
}
