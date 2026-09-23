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

use osal_rs::utils::{Bytes, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::Config;
use crate::apps::display::check::Check;
use crate::apps::screen_route::ScreenId;
use crate::drivers::date_time::DateTime;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};

pub(super) struct ScreenDaylightSavingTime {
    enable_dst: Check,
}

impl ScreenRoute<ScreenId> for ScreenDaylightSavingTime {

    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::DaylightSavingTime
    }

    fn draw(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
    
        match self.enable_dst.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Enable DST?"),
            ScreenParam::Check(Config::shared().get_daylight_saving_time().is_enabled()),
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(param) => {
                
                match param {
                    ScreenParam::Check(value) => {
                        Self::save(value)?;
                    }
                    _ => {}
                }
                
                Ok(Nav::Pop)
            }
            Answer::Cancelled => Ok(Nav::Pop),
        }
    }
}

impl ScreenDaylightSavingTime {

    fn save(enabled: bool) -> Result<()> {
        DateTime::set_daylight_saving_time(enabled);
        Config::shared().get_daylight_saving_time().set_enabled(enabled);
        Config::shared().apply_daylight_saving_time();
        Config::save()?;
        Ok(())
    }

    pub(super) const fn new() -> Self {
        Self {
            enable_dst: Check::new(),
        }
    }
}
