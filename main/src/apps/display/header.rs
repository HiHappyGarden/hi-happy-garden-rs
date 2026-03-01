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

use alloc::format;
use alloc::sync::Arc;
use osal_rs::{log_error};
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::Result;
use crate::assets::font_5x8::FONT_5X8;
use crate::assets::ic_wifi_excellent::IC_WIFI_EXCELLENT;
use crate::assets::ic_wifi_fair::IC_WIFI_FAIR;
use crate::assets::ic_wifi_good::IC_WIFI_GOOD;
use crate::assets::ic_wifi_no_signal::IC_WIFI_NO_SIGNAL;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};

use crate::traits::rtc::RTC;
use crate::traits::wifi::RSSIStatus::{self, *};

pub(super) struct Header<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{
    rtc: Arc<&'static dyn RTC>,
    lcd: Arc<Mutex<T>>,
}

impl<T> Header<T> 
where T: LCDDisplayFn + Sync + Send + Clone + 'static
{

    const HEIGHT: u8 = 10;

    pub(super) fn new(rtc: Arc<&'static dyn RTC>, lcd: Arc<Mutex<T>>) -> Self {
        Self {
            rtc,
            lcd,
        }
    }

    pub(super) fn draw(&mut self, rssi: RSSIStatus) -> Result<()> {
        

        let mut lcd = self.lcd.lock().unwrap();

        let (display_width, _) = lcd.get_size();

        lcd.draw_rect(0, 0, display_width, Self::HEIGHT, LCDWriteMode::REMOVE)?;

        match rssi {
            Unknown => {},
            Excellent => lcd.draw_bitmap_image(3, 0, IC_WIFI_EXCELLENT.0, IC_WIFI_EXCELLENT.1, &IC_WIFI_EXCELLENT.2, LCDWriteMode::ADD)?,
            Good => lcd.draw_bitmap_image(3, 0, IC_WIFI_GOOD.0, IC_WIFI_GOOD.1, &IC_WIFI_GOOD.2, LCDWriteMode::ADD)?,
            Fair | Weak => lcd.draw_bitmap_image(3, 0, IC_WIFI_FAIR.0, IC_WIFI_FAIR.1, &IC_WIFI_FAIR.2, LCDWriteMode::ADD)?,
            NoSignal => lcd.draw_bitmap_image(3, 0, IC_WIFI_NO_SIGNAL.0, IC_WIFI_NO_SIGNAL.1, &IC_WIFI_NO_SIGNAL.2, LCDWriteMode::ADD)?,
        }
        
        lcd.draw_rect(0, Self::HEIGHT + 1, display_width, 1, LCDWriteMode::ADD)?;


        if self.rtc.is_to_synch() {
            self.rtc.get_timestamp().map(|timestamp| {
                let datetime = if let Ok(dt) = DateTime::from_timestamp_locale(timestamp, true) {
                    dt
                } else {
                    log_error!("Header", "Failed to convert RTC timestamp to datetime");
                    return;
                };

                let now = format!("{:02}/{:02}/{:02} {:02}:{:02}:{:02}", datetime.mday, datetime.month, datetime.year, datetime.hour, datetime.minute, datetime.second);
                lcd.draw_str(&now, display_width - (now.len() as u8 * 5) - 5, 1, &FONT_5X8).unwrap_or_else(|e| {
                    log_error!("Header", "Failed to draw time on LCD: {}", e);
                });
            }).unwrap_or_else(|e| {
                log_error!("Header", "Failed to get timestamp from RTC: {}", e);
            });
        }

        Ok(())
    }
}
