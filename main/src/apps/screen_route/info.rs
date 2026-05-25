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

use core::any::Any;
use core::sync::atomic::{AtomicBool, Ordering};

use alloc::sync::Arc;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::config::Config;
use crate::apps::display::text::Text;
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::wifi::Wifi;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute};

static BACK: AtomicBool = AtomicBool::new(false);

pub struct ScreenInfo {
    text: Text,
}

impl ScreenRoute for ScreenInfo {
    #[allow(unused)]
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[allow(unused)]
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits, 
        _status_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
        
    ) -> Result<()> {

        let mut text = Bytes::<DISPLAY_INPUT_MAX_SIZE>::new();

        if Config::shared().get_wifi_config().is_enabled() {
            text.format(format_args!("Ip Address|{}", Wifi::get_ip_address()));
        } else {
            text.append_str("Wifi: Disabled");
        }

        self.text.draw(
                    lcd, 
                    display_signal, 
                    rtc, 
                    &text, 
                    ScreenParam::<u16>::default(), 
                    Some(|_, _| {
                        BACK.store(true, core::sync::atomic::Ordering::Relaxed);
                    })
                )?;

        if BACK.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(Error::ReturnWithCode(1))
        }
    }
}

impl ScreenInfo {
    pub fn new() -> Self {
        Self {
            text: Text::new(),
        }
    }
}