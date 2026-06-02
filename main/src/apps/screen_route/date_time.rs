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
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::display::commons::get_datetime_from_rtc;
use crate::apps::display::date::Date;
use crate::apps::display::time::Time;
use crate::apps::signals::display::DisplayFlag;
use crate::apps::signals::error::ErrorFlag;
use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute};

static mut FSM_STATE: FSMState = FSMState::Date;
static UPDATE_DRAW: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Date,
    Time,
    Save,
    End,
}

pub(super) struct ScreenDateTime {
    date: Date,
    time: Time,
}

impl ScreenRoute for ScreenDateTime {
    fn draw(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        _status_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        if UPDATE_DRAW.load(Ordering::SeqCst) {
            UPDATE_DRAW.store(false, Ordering::SeqCst);
            *display_signal |= DisplayFlag::Draw as u32;
        }

        match unsafe { *&raw const FSM_STATE } {
            FSMState::Date => self.draw_date_state(lcd, display_signal, rtc)?,
            FSMState::Time => self.draw_time_state(lcd, display_signal, rtc)?,
            FSMState::Save => self.draw_save_state(rtc)?,
            FSMState::End => return Ok(())
            
        }

        Err(Error::ReturnWithCode(1))
    }

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
}

impl ScreenDateTime {
    fn draw_date_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let date_time = get_datetime_from_rtc!(rtc, ErrorFlag::DateTime);
        let mut param = ScreenParam::default();
        param.date_time = Some(date_time);

        self.date.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Set Date"),
            param,
            Some(|_, confirmed| {
                unsafe { FSM_STATE = if confirmed { FSMState::Time } else { FSMState::End }; }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_time_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let date_time = get_datetime_from_rtc!(rtc, ErrorFlag::DateTime);
        let mut param = ScreenParam::default();
        param.date_time = Some(date_time);

        self.time.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Set Time"),
            param,
            Some(|_, confirmed| {
                unsafe { FSM_STATE = if confirmed { FSMState::Save } else { FSMState::Date }; }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_save_state(&mut self, rtc: &Arc<Mutex<dyn RTC + 'static>>) -> Result<()> {
        let DateTime { year, month, mday, wday, .. } = self.date.get_value()?;
        let DateTime { hour, minute, second, .. } = self.time.get_value()?;
        let date_time = DateTime::new(year, month, wday, mday, hour, minute, second)?;
        rtc.lock()?.set_timestamp(date_time.to_timestamp())?;

        unsafe { FSM_STATE = FSMState::End; }
        UPDATE_DRAW.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub(super) const fn new() -> Self {
        Self {
            date: Date::new(),
            time: Time::new(),
        }
    }
}
