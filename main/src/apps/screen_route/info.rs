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

use crate::apps::config::Config;
use crate::apps::display::text::Text;
use crate::apps::wifi::Wifi;
use crate::apps::signals::display::DisplayFlag;
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::screen_route::ScreenId;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute, ScreenRouteCtx, Nav};
use crate::traits::rtc::RTC;
use alloc::sync::Arc;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Result};


pub(super) struct ScreenInfo {
    text: Text
}


impl ScreenRoute<'_, ScreenId> for ScreenInfo {
    fn id(&self) -> ScreenId {
        ScreenId::Info
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {
    

         let mut text = Bytes::<DISPLAY_INPUT_MAX_SIZE>::new();

        if Config::shared().get_wifi_config().is_enabled() {
            text.format(format_args!("Ip Address|{}", Wifi::get_ip_address()));
        } else {
            text.append_str("Wifi: Disabled");
        }

        
       self.text.draw(
            screen_route_ctx.lcd,
            screen_route_ctx.display_signal,
            screen_route_ctx.rtc,
            &text,
            ScreenParam::<u16>::default()
        )?;

        if *screen_route_ctx.display_signal & DisplayFlag::EncoderButtonReleased as u32 != 0 {
            // The router builds the screen, so the menu stays independent from it.
            Ok(Nav::PushId(ScreenId::Info))
        } else {
            Ok(Nav::Stay)
        }
    }
}

impl ScreenInfo {
    pub(super) fn new() -> Self {
        Self {
            text: Text::new(),
        }
    }
}   