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

use core::clone;

use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result};

use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};
use crate::traits::integer::Integer;

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::{signals::display::request_redraw};
use crate::apps::sprinkler::schedule::{Day, Month, Schedule, ScheduleController};
use crate::apps::display::select::Select;
use crate::apps::display::number::Number;
use crate::apps::display::input::Input;
use crate::apps::display::check::{self, Check}; 

enum FSMState {
    Description,
    Minute,
    Hour,
    Days,
    Month,
    Zones,
    Status,
}



pub(super) struct ScreenScheduleDetail {
    index: usize,
    fsm_state: FSMState,

    description: Input,

    minute: Number<u8>,
    hour: Number<u8>,

    /// day of week from 0x01 to 0x40 or NOT_SET (0) for every day, otherwise bitmask of Day
    days: u8,
    fsm_days: Day,

    /// month, values allowed 0x01 to 0x0800 or NOT_SET (0) for every month
    month: u16,
    fsm_month: Month,

    check: Check,
    zones: Number<u8>, 
    status: Select<2>
}

impl ScreenRoute for ScreenScheduleDetail {
    fn id() -> &'static str {
        "ScreenScheduleDetail"
    }

    fn renderize(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav> {

        match self.fsm_state {
            FSMState::Minute    => self.draw_minute_state(screen_route_ctx),
            FSMState::Hour      => self.draw_hour_state(screen_route_ctx),
            FSMState::Days      => self.draw_days_state(screen_route_ctx),
            FSMState::Month     => self.draw_month_state(screen_route_ctx),
            FSMState::Description => self.draw_description_state(screen_route_ctx),
            FSMState::Zones     => self.draw_zones_state(screen_route_ctx),
            FSMState::Status    => self.draw_status_state(screen_route_ctx),
        }
    }
}

impl ScreenScheduleDetail {
    pub(super)const fn new(index: usize) -> Result<Self> {
        if index > ScheduleController::SIZE {
            return Err(Error::OutOfIndex)
        }
        Ok(Self { 
            index,
            fsm_state: FSMState::Minute,
            description: Input::new(),
            minute: Number::new_with_range(0..=59),
            hour: Number::new_with_range(0..=23),
            days: Schedule::NOT_SET,
            fsm_days: Day::Monday,
            month: Schedule::NOT_SET as u16,
            fsm_month: Month::January,
            check: Check::new(),
            zones: Number::new_with_range(0..=3),
            status: Select::new(),
        })
    }

    fn draw_description_state(&mut self, ScreenRouteCtx{lcd, display_signal, rtc, .. } : &mut ScreenRouteCtx<'_>) -> Result<Nav> {
        let des = ScheduleController::default()[self.index];

        let answer = self.description.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(des.description),
                ScreenParam::Input { 
                    value: ScheduleController::shared()[self.index].description,
                    secret_mode: false 
                }
            )?;

        Ok(self.step(display_signal, answer, FSMState::Minute, FSMState::Description))
    }

    fn draw_minute_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        let answer = self.minute.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Minutes"),
                ScreenParam::Number(self.minute.get_value()?)
            )?;

        Ok(self.step(display_signal, answer, FSMState::Hour, FSMState::Description))
    }

    fn draw_hour_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        let answer = self.hour.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Hours"),
                ScreenParam::Number(self.hour.get_value()?)
            )?;

        Ok(self.step(display_signal, answer, FSMState::Days, FSMState::Minute))
    }

    fn draw_days_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        
        let mut title = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Day - ");
        title.append_str(self.fsm_days.into());


        let answer = self.check.draw(
                *lcd,
                display_signal,
                rtc,
                &title,
                ScreenParam::Check(self.fsm_days.check(self.days))
            )?;

        
        let next;
        let back;
        if let Answer::Confirmed(ScreenParam::Check(check)) = answer {

            match self.fsm_days {
                Day::Sunday => {
                    next = FSMState::Days;
                    back = FSMState::Hour;
                }
                Day::Monday | Day::Tuesday | Day::Wednesday | Day::Thursday | Day::Friday => {
                    next = FSMState::Days;
                    back = FSMState::Days;
                }
                Day::Saturday => {
                    next = FSMState::Month;
                    back = FSMState::Days;
                }
            }

            
        } else {
            next = FSMState::Days;
            back = FSMState::Hour;
        }

        Ok(self.step(display_signal, answer, next, back))
    }

    fn draw_month_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        // Implement the rendering logic for the month state here
        Ok(Nav::Stay)
    }

    fn draw_zones_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        // Implement the rendering logic for the zones state here
        Ok(Nav::Stay)
    }

    fn draw_status_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        // Implement the rendering logic for the status state here
        Ok(Nav::Stay)
    }

    #[inline]
    fn set_state(&mut self, display_signal: &mut EventBits, next: FSMState) {
        self.fsm_state = next;
        request_redraw(display_signal);
    }


    fn step<N, const N_SELECTS: usize>(&mut self, display_signal: &mut EventBits, answer: Answer<N, N_SELECTS>, confirmed: FSMState, cancelled: FSMState) -> Nav
    where N: Integer
    {
        match answer {
            Answer::Pending => {}
            Answer::Confirmed(_) => self.set_state(display_signal, confirmed),
            Answer::Cancelled => self.set_state(display_signal, cancelled),
        }
        Nav::Stay
    }


}