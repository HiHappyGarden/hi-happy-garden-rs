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

use alloc::sync::Arc;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::display::commons::get_datetime_from_rtc;
use crate::apps::display::date::Date;
use crate::apps::display::time::Time;
use crate::apps::screen_route::{ScreenId, request_redraw};
use crate::apps::signals::error::ErrorFlag;
use crate::drivers::date_time::DateTime;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Date,
    Time,
}

pub(super) struct ScreenDateTime {
    fsm_state: FSMState,
    date: Date,
    time: Time,
}

impl ScreenRoute<ScreenId> for ScreenDateTime {

    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::DateTime
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        match self.fsm_state {
            FSMState::Date => self.draw_date_state(screen_route_ctx),
            FSMState::Time => self.draw_time_state(screen_route_ctx),
        }
    }
}

impl ScreenDateTime {

    #[inline]
    fn set_state(&mut self, display_signal: &mut EventBits, next: FSMState) {
        self.fsm_state = next;
        request_redraw(display_signal);
    }

    fn draw_date_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::default();
        param.date_time = Some(get_datetime_from_rtc!(rtc, ErrorFlag::DateTime));

        match self.date.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Set Date"),
            param,
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.set_state(display_signal, FSMState::Time);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => Ok(Nav::Pop),
        }
    }

    fn draw_time_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::default();
        param.date_time = Some(get_datetime_from_rtc!(rtc, ErrorFlag::DateTime));

        match self.time.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Set Time"),
            param,
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.save(rtc)?;
                Ok(Nav::Pop)
            }
            Answer::Cancelled => {
                self.set_state(display_signal, FSMState::Date);
                Ok(Nav::Stay)
            }
        }
    }

    fn save(&self, rtc: &Arc<Mutex<dyn RTC + 'static>>) -> Result<()> {
        let DateTime { year, month, mday, wday, .. } = self.date.get_value()?;
        let DateTime { hour, minute, second, .. } = self.time.get_value()?;
        let date_time = DateTime::new(year, month, wday, mday, hour, minute, second)?;
        rtc.lock()?.set_timestamp(date_time.to_timestamp())?;
        Ok(())
    }

    pub(super) const fn new() -> Self {
        Self {
            fsm_state: FSMState::Date,
            date: Date::new(),
            time: Time::new(),
        }
    }
}
