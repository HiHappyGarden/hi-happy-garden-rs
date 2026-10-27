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
use osal_rs::println;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::display::text::Text;
use crate::apps::sprinkler::commons::Status;
use crate::apps::sprinkler::zone::ZoneRelay::Relay0;
use crate::apps::sprinkler::schedule::ScheduleZone;
use crate::apps::sprinkler::zone::{ZoneController, ZoneRelay};
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};
use crate::traits::integer::Integer;

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::{signals::display::request_redraw};
use crate::apps::sprinkler::schedule::{Day, Month, Schedule, ScheduleController};
use crate::apps::display::select::Select;
use crate::apps::display::number::Number;
use crate::apps::display::input::Input;
use crate::apps::display::check::Check;

enum FSMState {
    Description,
    Minute,
    Hour,
    DaysLabel,
    Days,
    MonthLabel,
    Month,
    ZoneLabel,
    Zone,
    Status,
}



pub(super) struct ScreenScheduleDetail {
    index: usize,
    schedule: &'static Schedule,

    fsm_state: FSMState,

    description: Input,

    minute: Number<u8>,
    hour: Number<u8>,

    text: Text,
    check: Check,

    /// day of week from 0x01 to 0x40 or NOT_SET (0) for every day, otherwise bitmask of Day
    days: u8,
    fsm_day: Day,

    /// month, values allowed 0x01 to 0x0800 or NOT_SET (0) for every month
    months: u16,
    fsm_month: Month,

    number: Number<u8>,
    zones: [ScheduleZone; ZoneController::SIZE],
    fsm_zone: ZoneRelay,

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
            FSMState::DaysLabel => self.draw_label_state(screen_route_ctx, "Select watering|days", FSMState::Days, FSMState::Hour),
            FSMState::Days      => self.draw_days_state(screen_route_ctx),
            FSMState::MonthLabel => self.draw_label_state(screen_route_ctx, "Select watering|months", FSMState::Month, FSMState::Days),
            FSMState::Month     => self.draw_month_state(screen_route_ctx),
            FSMState::ZoneLabel => self.draw_label_state(screen_route_ctx, "Select watering|zones", FSMState::Zone, FSMState::Month),
            FSMState::Zone     => self.draw_zones_state(screen_route_ctx),
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
            text: Text::new(),
            check: Check::new(),
            fsm_day: Day::Monday,
            months: schedule.months,
            fsm_month: Month::January,
            
            number: Number::new_with_range(0..=255),
            zones: schedule.zones,
            fsm_zone: ZoneRelay::Relay0,

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
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Start at minute"),
                ScreenParam::Number(self.schedule.minute)
            )?;

        Ok(self.step(display_signal, answer, FSMState::Hour, FSMState::Description))
    }

    fn draw_hour_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        let answer = self.hour.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Start at hours"),
                ScreenParam::Number(self.schedule.hour)
            )?;

        self.fsm_day = Day::Monday;

        Ok(self.step(display_signal, answer, FSMState::DaysLabel, FSMState::Minute))
    }

    fn draw_label_state(&mut self, ScreenRouteCtx{lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>, text: &str, confirmed: FSMState, cancelled: FSMState) -> Result<Nav> {
        let answer = self.text.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(text),
                ScreenParam::None
            )?;

        match confirmed {
            FSMState::Days => self.fsm_day = Day::Monday,
            FSMState::Month => self.fsm_month = Month::January,
            FSMState::Zone => self.fsm_zone = ZoneRelay::Relay0,
            _ => {}
        }
        

        Ok(self.step(display_signal, answer, confirmed, cancelled))
    }

    fn draw_days_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        
        let mut title = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Enable ");
        title.append_str(
            <Day as Into<&str>>::into(self.fsm_day)
                .to_lowercase()
                .as_str()
            );


        let value = self.fsm_day.check(self.days);

        println!("Current day value: {} self.days: {}", value, self.days);

        let answer = self.check.draw(
                *lcd,
                display_signal,
                rtc,
                &title,
                ScreenParam::Check(value)
            )?;

        
        let next;
        let back;
        if let Answer::Confirmed(ScreenParam::Check(check)) = answer {

            if check {
                self.days |= self.fsm_day as u8;
            } else {
                self.days &= !(self.fsm_day as u8);
            }
            self.check.drop_value();

            match self.fsm_day {
                Day::Monday => {
                    next = FSMState::Days;
                    back = FSMState::DaysLabel;
                    self.fsm_day = Day::Tuesday;
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
                    next = FSMState::Days;
                    back = FSMState::Days;
                    self.fsm_day = Day::Sunday;
                }
                Day::Sunday => {
                    next = FSMState::MonthLabel;
                    back = FSMState::Days;
                    self.fsm_day = Day::Monday;
                }
            }

            
        } else {
            next = FSMState::Days;
            back = FSMState::DaysLabel;
        }

        self.fsm_month = Month::January;

        Ok(self.step(display_signal, answer, next, back))
    }

    fn draw_month_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
 
        let mut title = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Enable ");
        title.append_str(
            <Month as Into<&str>>::into(self.fsm_month)
                .to_lowercase()
                .as_str()
            );


        let value = self.fsm_month.check(self.months);

        let answer = self.check.draw(
                *lcd,
                display_signal,
                rtc,
                &title,
                ScreenParam::Check(value)
            )?;

        
        let next;
        let back;
        if let Answer::Confirmed(ScreenParam::Check(check)) = answer {

            if check {
                self.months |= self.fsm_month as u16;
            } else {
                self.months &= !(self.fsm_month as u16);
            }
            self.check.drop_value();

            match self.fsm_month {
                Month::January => {
                    next = FSMState::Month;
                    back = FSMState::MonthLabel;
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
                    next = FSMState::ZoneLabel;
                    back = FSMState::Month;
                    self.fsm_month = Month::January;
                }
            }

            
        } else {
            next = FSMState::Month;
            back = FSMState::MonthLabel;
        }


        self.fsm_zone = Relay0;

        Ok(self.step(display_signal, answer, next, back))
    }

    fn draw_zones_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {

        let mut title = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Set ");
        title.append(&ZoneController::shared()[self.fsm_zone as usize].description);
        title.append_str(" watering time");


        let value = self.zones[self.fsm_zone as usize].minutes;

        let answer = self.number.draw(
                *lcd,
                display_signal,
                rtc,
                &title,
                ScreenParam::Number(value)
            )?;

        
        let next;
        let back;
        if let Answer::Confirmed(ScreenParam::Number(number)) = answer {

            // if number > 0 {
            self.zones[self.fsm_zone as usize] = ScheduleZone { relay: self.fsm_zone, minutes: number };
            self.number.drop_value();

            match self.fsm_zone {
                ZoneRelay::Relay0 => {
                    next = FSMState::Zone;
                    back = FSMState::ZoneLabel;
                    self.fsm_zone = ZoneRelay::Relay1;
                }
                ZoneRelay::Relay1 => {
                    next = FSMState::Zone;
                    back = FSMState::Zone;
                    self.fsm_zone = ZoneRelay::Relay2;
                }
                ZoneRelay::Relay2 => {
                    next = FSMState::Zone;
                    back = FSMState::Zone;
                    self.fsm_zone = ZoneRelay::Relay3;
                }
                ZoneRelay::Relay3 => {
                    next = FSMState::Status;
                    back = FSMState::Zone;
                    self.fsm_zone = ZoneRelay::Relay0;
                }
            }

            
        } else {
            next = FSMState::Zone;
            back = FSMState::ZoneLabel;
        }

        Ok(self.step(display_signal, answer, next, back))
    }

    fn draw_status_state(&mut self, ScreenRouteCtx{lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        let mut selects: [(Bytes<{DISPLAY_INPUT_MAX_SIZE}>, bool); 2] = [
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(Status::ACTIVE.as_str()), false),
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(Status::UNACTIVE.as_str()), false),
        ];

        selects[0].1 = self.schedule.status == Status::ACTIVE;
        selects[1].1 = self.schedule.status == Status::UNACTIVE;

        
        let answer = self.status.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Status"),
                ScreenParam::Selects(selects)
            )?;

        match answer {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(answer) => {
                match answer {
                    ScreenParam::Selects(selected) => {

                        let status  = selected
                            .iter()
                            .find(|(_, selected)| *selected)
                            .ok_or(Error::OutOfIndex)?
                            .0
                            .clone();

                        let new_schedule = Schedule {
                            minute: self.minute.get_value()?,
                            hour: self.hour.get_value()?,
                            days: self.days,
                            months: self.months,
                            description: self.description.get_value()?,
                            zones: self.zones,
                            status: Status::from(status.as_str()),
                        };

                        ScheduleController::shared()[self.index] = new_schedule;
                        
                        ScheduleController::shared().save()?;

                        Ok(Nav::Pop(true))
                    }
                    _ => Err(Error::InvalidType),
                }
            }
            Answer::Cancelled => {
                self.set_state(display_signal, FSMState::Zone);
                Ok(Nav::Stay)
            }
        }
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