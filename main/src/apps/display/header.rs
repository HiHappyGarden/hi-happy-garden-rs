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

use alloc::format;

use osal_rs::os::types::EventBits;
use osal_rs::log_error;
use osal_rs::utils::Result;

use crate::apps::signals::display::DisplayFlag;
use crate::apps::signals::error::{ErrorFlag, ErrorSignal};

use crate::assets::font_5x8::FONT_5X8;
use crate::assets::ic_wifi_excellent::IC_WIFI_EXCELLENT;
use crate::assets::ic_wifi_fair::IC_WIFI_FAIR;
use crate::assets::ic_wifi_good::IC_WIFI_GOOD;
use crate::assets::ic_wifi_no_signal::IC_WIFI_NO_SIGNAL;

use crate::drivers::date_time::DateTime;

use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::signal::Signal;
use crate::traits::wifi::RSSIStatus::{self, *};

pub(super) struct Header
{
    date_time: DateTime,
    rssi_status: RSSIStatus,
}

impl Header
{


    pub(super) fn new() -> Self {
        Self {
            date_time: DateTime::default(),
            rssi_status: RSSIStatus::Unknown,
        }
    }

    pub(super) fn draw(&mut self, lcd: &mut impl LCDDisplayFn, signals: &mut EventBits, date_time: &DateTime, wifi_enabled: bool) -> Result<()> {
        
        let rssi = match RSSIStatus::from_bites( (*signals >> 6) as u8 ) {
            Ok(status) => status,
            Err(_) => RSSIStatus::Unknown,
        };


        let mut redraw_needed = false;

        if date_time.hour != self.date_time.hour || date_time.minute != self.date_time.minute {
            redraw_needed = true;
        }

        if rssi != RSSIStatus::Unknown && self.rssi_status != rssi {
            self.rssi_status = rssi;
            redraw_needed = true;
        }

        if !redraw_needed {
            return Ok(()); // No need to redraw if nothing has changed
        }

        self.date_time = date_time.clone();

        let (display_width, _) = lcd.get_size();

        let header_height = lcd.get_header_height() - 1;

        lcd.draw_rect(0, 0, display_width, header_height, LCDWriteMode::REMOVE)?;

        match self.rssi_status {
            Unknown => if wifi_enabled {
                //lcd.draw_bitmap_image(3, 0, IC_WIFI_NO_SIGNAL.0, IC_WIFI_NO_SIGNAL.1, &IC_WIFI_NO_SIGNAL.2, LCDWriteMode::ADD)?;
            }
            Excellent => lcd.draw_bitmap_image(3, 0, IC_WIFI_EXCELLENT.0, IC_WIFI_EXCELLENT.1, &IC_WIFI_EXCELLENT.2, LCDWriteMode::ADD)?,
            Good => lcd.draw_bitmap_image(3, 0, IC_WIFI_GOOD.0, IC_WIFI_GOOD.1, &IC_WIFI_GOOD.2, LCDWriteMode::ADD)?,
            Fair | Weak => lcd.draw_bitmap_image(3, 0, IC_WIFI_FAIR.0, IC_WIFI_FAIR.1, &IC_WIFI_FAIR.2, LCDWriteMode::ADD)?,
            NoSignal => lcd.draw_bitmap_image(3, 0, IC_WIFI_NO_SIGNAL.0, IC_WIFI_NO_SIGNAL.1, &IC_WIFI_NO_SIGNAL.2, LCDWriteMode::ADD)?,
        }
        
        lcd.draw_rect(0, header_height, display_width, 1, LCDWriteMode::ADD)?;


        if date_time.is_valid() {
            let now = format!("{:04}-{:02}-{:02}  {:02}:{:02}", date_time.year, date_time.month, date_time.mday, date_time.hour, date_time.minute);
            lcd.draw_str(&now, display_width - (now.len() as u8 * 5) - 5, 1, &FONT_5X8).unwrap_or_else(|e| {
                log_error!("Header", "Failed to draw time on LCD: {}", e);
                ErrorSignal::set(ErrorFlag::Display.into());
            });
        }

        *signals |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn 
        
        Ok(())
    }
}
