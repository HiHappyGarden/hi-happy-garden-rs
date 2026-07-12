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

use at_parser_rs::{Args, AtError, AtResult};
use at_parser_rs::context::AtContext;
use osal_rs::{access_static_option, log_info};
use osal_rs::os::{RawMutex, RawMutexGuard};
use osal_rs::utils::{Bytes, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::parser::Parser;
use crate::apps::sprinkler::zone::{ZoneController, ZoneRelay};
use crate::apps::utils::deserialize_file;
use crate::drivers::date_time::DateTime;
use crate::drivers::platform::FS_CONFIG_DIR;
use crate::traits::state::Initializable;
use super::commons::Status;

static mut SHARED: ScheduleController = ScheduleController { schedules: [
    Schedule::new(),
    Schedule::new(),
    Schedule::new(),
    Schedule::new()
]};


static mut MUTEX: Option<RawMutex> = None;


const APP_TAG: &str = "SchedulerController";

 #[allow(dead_code)]
 #[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(in crate::apps) enum Day {
    Sunday = 0x01,
    Monday = 0x02,
    Tuesday = 0x04,
    Wednesday = 0x08,
    Thursday = 0x10,
    Friday = 0x20,
    Saturday = 0x40
} 

impl Day {
    fn map(value: u8) -> [Option<Self>; 7] {
        use Day::*;
        
        let mut ret = [None; 7];

        for idx in 0u8..7 {
            if value & (1 << idx) > 0 {
                ret[idx as usize] = match idx {
                    0 => Some(Sunday),
                    1 => Some(Monday),
                    2 => Some(Tuesday),
                    3 => Some(Wednesday),
                    4 => Some(Thursday),
                    5 => Some(Friday),
                    6 => Some(Saturday),
                    _ => None
                }
            } else {
                ret[idx as usize] = None;
            }
        }

        ret
    }
}

impl From<Day> for u8 {
    fn from(value: Day) -> Self {
        match value {
            Day::Sunday => 0,
            Day::Monday => 1,
            Day::Tuesday => 2,
            Day::Wednesday => 3,
            Day::Thursday => 4,
            Day::Friday => 5,
            Day::Saturday => 6
        }
    }
}

 #[allow(dead_code)]
 #[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(in crate::apps) enum Month {
    January = 0x0001,
    February = 0x0002,
    March = 0x0004,
    April = 0x0008,
    May = 0x0010,
    June = 0x0020,
    July = 0x0040,
    August = 0x0080,
    September = 0x0100,
    October = 0x0200,
    November = 0x0400,
    December = 0x0800,
}

impl From<Month> for u8 {
    fn from(value: Month) -> Self {
        match value {
            Month::January => 0,
            Month::February => 1,
            Month::March => 2,
            Month::April => 3,
            Month::May => 4,
            Month::June => 5,
            Month::July => 6,
            Month::August => 7,
            Month::September => 8,
            Month::October => 9,
            Month::November => 10,
            Month::December => 11
        }
    }
}

impl Month {
    fn map(value: u16) -> [Option<Self>; 12] {
        use Month::*;
        
        let mut ret = [None; 12];

        for idx in 0u16..12 {
            if value & (1 << idx) > 0 {
                ret[idx as usize] = match idx {
                    0 => Some(January),
                    1 => Some(February),
                    2 => Some(March),
                    3 => Some(April),
                    4 => Some(May),
                    5 => Some(June),
                    6 => Some(July),
                    7 => Some(August),
                    8 => Some(September),
                    9 => Some(October),
                    10 => Some(November),
                    11 => Some(December),
                    _ => None
                }
            } else {
                ret[idx as usize] = None;
            }
        }

        ret
    }
}



#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct Schedule {

    ///  minute, values allowed 1 - 60 or NOT_SET (0) for every minute real value is minute - 1
    pub minute: u8,

    /// hour, values allowed 1 - 24 or NOT_SET (0) for every hour real value is hour - 1
    pub hour: u8,

    /// day of week from 0x01 to 0x40 or NOT_SET (0) for every day, otherwise bitmask of Day
    pub days: u8,

    /// month, values allowed 0x01 to 0x0800 or NOT_SET (0) for every month
    pub month: u16,

    /// description 
    pub description: Bytes<DISPLAY_INPUT_MAX_SIZE>,

    /// zones associated to the schedule
    pub zones: [Option<ZoneRelay>; ZoneController::SIZE],

    /// status of the schedule
    pub status: Status
}

impl Schedule {
    pub(super) const NOT_SET: u8 = 0x00;

    pub(super) const fn new() -> Self {
        Self {
            minute: 0,
            hour: 0,
            days: Schedule::NOT_SET,
            month: Schedule::NOT_SET as u16,
            description: Bytes::new(),
            zones: [
                None,
                None,
                None,
                None
            ],
            status: Status::UNACTIVE
        }
    }

    pub(in super) fn executable(&self, now: &DateTime) -> bool {
        if self.status == Status::RUN {
            return false;
        }

        let mut check = [true; 4];

        if self.month != Schedule::NOT_SET as u16 {
            let months = Month::map(self.month);
            for month in months.iter() {
                check[0] = false;
                if let Some(m) = month {
                    if <Month as Into<u8>>::into(*m) == now.month {
                        check[0] = true;
                        break;
                    }
                }
            }
        } else {
            check[0] = true;
        }

        if self.days != Schedule::NOT_SET {
            let days = Day::map(self.days);
            for day in days.iter() {
                check[1] = false;
                if let Some(d) = day {
                    if <Day as Into<u8>>::into(*d) == now.wday {
                        check[1] = true;
                        break;
                    }
                }
            }
        } else {
            check[1] = true;
        }

        if self.hour != Schedule::NOT_SET {
            check[2] = self.hour - 1 == now.hour;
        } else {
            check[2] = true;
        }

        if self.minute != Schedule::NOT_SET {
            check[3] = self.minute - 1 == now.minute;
        } else {
            check[3] = true;
        }

        check.iter().all(|&x| x)
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct ScheduleController {
    schedules: [Schedule; ScheduleController::SIZE]
}


impl Initializable for ScheduleController {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init Schedule");

        let _lock = RawMutexGuard::acquire(access_static_option!(MUTEX));

        let mut count = 0u8;
        unsafe {
            for Schedule{description, status,  .. } in &mut *&raw mut SHARED.schedules {
                description.format(format_args!("Schedule {count}"));
                *status = Status::UNACTIVE;
                count += 1;
            }
        }

        *self = deserialize_file::<ScheduleController>(unsafe { &*&raw const MUTEX }, APP_TAG, FS_CONFIG_DIR, ScheduleController::FILE_NAME)?;

        Ok(())
    }
}

impl<'a> IntoIterator for &'a mut ScheduleController {
    type Item = &'a mut Schedule;
    type IntoIter = core::slice::IterMut<'a, Schedule>;

    fn into_iter(self) -> Self::IntoIter {
        self.schedules.iter_mut()
    }
}

impl AtContext<{Parser::CMD_SIZE}> for ScheduleController {
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, {Parser::CMD_SIZE}> {
        Err((at_response, AtError::NotSupported))
    }

    fn query(&mut self, at_response: &'static str) -> AtResult<'_, {Parser::CMD_SIZE}> {
        Err((at_response, AtError::NotSupported))
    }

    fn test(&mut self, at_response: &'static str) -> AtResult<'_, {Parser::CMD_SIZE}> {
        Err((at_response, AtError::NotSupported))
    }

    fn set(&mut self, at_response: &'static str, _args: Args) -> AtResult<'_, {Parser::CMD_SIZE}> {
        Err((at_response, AtError::NotSupported))
    }
}

impl ScheduleController {
    pub(in crate::apps) const SIZE: usize = 4;
    pub(in crate::apps) const AT_CMD: &'static str = "AT+SCH";
    pub(in crate::apps) const AT_RESP: &'static str = "+SCH: ";
    const FILE_NAME: &'static str = "schedules.json";

    pub(in crate::apps) fn shared() -> &'static mut Self {
        unsafe {
            if (*&raw const MUTEX).is_none() {
                MUTEX = match RawMutex::new() {
                    Ok(mutex) => Some(mutex),
                    Err(_) =>  panic!("MUTEX is not initialized",),
                }
            }
        }

        let _lock = RawMutexGuard::acquire(access_static_option!(MUTEX));
        unsafe { &mut *&raw mut SHARED }
    }

}
