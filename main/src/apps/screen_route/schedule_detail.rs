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


use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::sprinkler::zone::{ZoneController, ZoneRelay};
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
    schedule: &'static Schedule,

    fsm_state: FSMState,

    description: Input,

    minute: Number<u8>,
    hour: Number<u8>,

    /// day of week from 0x01 to 0x40 or NOT_SET (0) for every day, otherwise bitmask of Day
    days: u8,
    fsm_day: Day,

    /// month, values allowed 0x01 to 0x0800 or NOT_SET (0) for every month
    months: u16,
    fsm_month: Month,

    check: Check,
    zones: Select<{ZoneController::SIZE + 1}>,
    
    status: Select<2>
}

impl ScreenRoute for ScreenScheduleDetail {
    fn id() -> &'static str {
        "ScreenScheduleDetail"
    }

    fn renderize(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav> {

        match self.fsm_state {
            FSMState::Description => self.draw_description_state(screen_route_ctx),
            FSMState::Minute    => self.draw_minute_state(screen_route_ctx),
            FSMState::Hour      => self.draw_hour_state(screen_route_ctx),
            FSMState::Days      => self.draw_days_state(screen_route_ctx),
            FSMState::Month     => self.draw_month_state(screen_route_ctx),
            FSMState::Zones     => self.draw_zones_state(screen_route_ctx),
            FSMState::Status    => self.draw_status_state(screen_route_ctx),
        }
    }
}

impl ScreenScheduleDetail {
    pub(super) fn new(index: usize) -> Result<Self> {
        if index >= ScheduleController::SIZE {
            return Err(Error::OutOfIndex)
        }

        let schedule = &ScheduleController::shared()[index];

        Ok(Self { 
            index,
            schedule,
            fsm_state: FSMState::Description,
            description: Input::new(),
            minute: Number::new_with_range(0..=59),
            hour: Number::new_with_range(0..=23),
            days: schedule.days,
            fsm_day: Day::Monday,
            months: schedule.months,
            fsm_month: Month::January,
            check: Check::new(),
            zones: Select::new(),
            status: Select::new(),
        })
    }

    fn draw_description_state(&mut self, ScreenRouteCtx{lcd, display_signal, rtc, .. } : &mut ScreenRouteCtx<'_>) -> Result<Nav> {

        let answer = self.description.draw(
                *lcd,
                display_signal,
                rtc,
                &ScheduleController::default()[self.index].description,
                ScreenParam::Input { 
                    value: self.schedule.description,
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
                ScreenParam::Number(self.schedule.minute)
            )?;

        Ok(self.step(display_signal, answer, FSMState::Hour, FSMState::Description))
    }

    fn draw_hour_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        let answer = self.hour.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Hours"),
                ScreenParam::Number(self.schedule.hour)
            )?;

        self.fsm_day = Day::Monday;

        Ok(self.step(display_signal, answer, FSMState::Days, FSMState::Minute))
    }

    fn draw_days_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        
        let mut title = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Day - ");
        title.append_str(self.fsm_day.into());


        self.check.drop_value();

        let answer = self.check.draw(
                *lcd,
                display_signal,
                rtc,
                &title,
                ScreenParam::Check(self.fsm_day.check(self.days))
            )?;

        
        let next;
        let back;
        if let Answer::Confirmed(ScreenParam::Check(check)) = answer {

            if check {
                self.days |= self.fsm_day as u8;
            }

            match self.fsm_day {
                Day::Monday => {
                    next = FSMState::Days;
                    back = FSMState::Hour;
                    self.fsm_day = Day::Thursday;
                }
                Day::Tuesday => {
                    next = FSMState::Days;
                    back = FSMState::Days;
                    self.fsm_day = Day::Wednesday;
                }
                Day::Wednesday => {
                    next = FSMState::Days;
                    back = FSMState::Days;
                    self.fsm_day = Day::Thursday;
                }
                Day::Thursday => {
                    next = FSMState::Days;
                    back = FSMState::Days;
                    self.fsm_day = Day::Friday;
                }
                Day::Friday => {
                    next = FSMState::Days;
                    back = FSMState::Days;
                    self.fsm_day = Day::Saturday;
                }
                Day::Saturday => {
                    next = FSMState::Month;
                    back = FSMState::Days;
                    self.fsm_day = Day::Sunday;
                }
                Day::Sunday => {
                    next = FSMState::Month;
                    back = FSMState::Days;
                    self.fsm_day = Day::Monday;
                }
            }

            
        } else {
            next = FSMState::Days;
            back = FSMState::Hour;
        }

        self.fsm_month = Month::January;

        Ok(self.step(display_signal, answer, next, back))
    }

    fn draw_month_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
 
        let mut title = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Month - ");
        title.append_str(self.fsm_month.into());


        self.check.drop_value();

        let answer = self.check.draw(
                *lcd,
                display_signal,
                rtc,
                &title,
                ScreenParam::Check(self.fsm_month.check(self.months))
            )?;

        
        let next;
        let back;
        if let Answer::Confirmed(ScreenParam::Check(check)) = answer {

            if check {
                self.months |= self.fsm_month as u16;
            }

            match self.fsm_month {
                Month::January => {
                    next = FSMState::Month;
                    back = FSMState::Days;
                    self.fsm_month = Month::February;
                }
                Month::February => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::March;
                }
                Month::March => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::April;
                }
                Month::April => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::May;
                }
                Month::May => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::June;
                }
                Month::June => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::July;
                }
                Month::July => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::August;
                }
                Month::August => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::September;
                }
                Month::September => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::October;
                }
                Month::October => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::November;
                }
                Month::November => {
                    next = FSMState::Month;
                    back = FSMState::Month;
                    self.fsm_month = Month::December;
                }
                Month::December => {
                    next = FSMState::Zones;
                    back = FSMState::Month;
                    self.fsm_month = Month::January;
                }
            }

            
        } else {
            next = FSMState::Month;
            back = FSMState::Days;
        }

        Ok(self.step(display_signal, answer, next, back))
    }

    fn draw_zones_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {


         let mut zones = [
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("None"), false),
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(ZoneRelay::Relay0.as_str()), false),
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(ZoneRelay::Relay1.as_str()), false),
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(ZoneRelay::Relay2.as_str()), false),
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(ZoneRelay::Relay3.as_str()), false),
        ];


        for (idx, zone) in self.schedule.zones.iter().enumerate() {
            zones[idx + 1].1 = match zone {
                None => idx == 0,
                Some((relay, _)) => match relay {
                    ZoneRelay::Relay0 => idx == 1,
                    ZoneRelay::Relay1 => idx == 2,
                    ZoneRelay::Relay2 => idx == 3,
                    ZoneRelay::Relay3 => idx == 4,
                    _ => false,
                },
            };
        }

        let answer = self.zones.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Select zone"),
                ScreenParam::Selects(zones)
            )?;

        Ok(self.step(display_signal, answer, FSMState::Status, FSMState::Zones))
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