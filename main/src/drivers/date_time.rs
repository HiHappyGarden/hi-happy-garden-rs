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

#![allow(unused)]

use core::fmt::{Debug, Display};

use alloc::string::String;
use osal_rs::utils::{Error, Result};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    pub year: i32,     //starting from 1970, can be negative for dates before 1970
    pub month: u8,     // 1-12
    pub day: u8,       // 1-31
    pub hour: u8,      // 0-23
    pub minute: u8,    // 0-59
    pub second: u8,    // 0-59
    pub timezone: i16, // in minutes, e.g. +120 for UTC+2, -60 for UTC-1
    pub daylight_saving_time: bool, // true if daylight saving time is in effect
}

impl DateTime {
    pub const SECONDS_PER_MINUTE: i64 = 60;
    pub const SECONDS_PER_HOUR: i64 = 3600;
    pub const SECONDS_PER_DAY: i64 = 86400;


    /// New Input: year, month (1-12), day (1-31), hour (0-23), minute (0-59), second (0-59)
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Result<Self> {
        // Validate input
        if month < 1 || month > 12 {
            return Err(Error::Unhandled("Invalid month"));
        }
        if day < 1 || day > Self::days_in_month(month, year) {
            return Err(Error::Unhandled("Invalid day for given month/year"));
        }
        if hour > 23 {
            return Err(Error::Unhandled("Invalid hour"));
        }
        if minute > 59 {
            return Err(Error::Unhandled("Invalid minute"));
        }
        if second > 59 {
            return Err(Error::Unhandled("Invalid second"));
        }

        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            timezone: 0,
            daylight_saving_time: false,
        })
    }

    /// Returns true if the year is a leap year
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Returns the number of days in a given month (accounting for leap year)
    fn days_in_month(month: u8, year: i32) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0, // invalid
        }
    }

    /// Creates a Time from a Unix timestamp (UTC)
    /// Input: Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
    /// Output: Result<Time>
    pub fn from_timestamp(timestamp: i64) -> Result<Self> {
        // Split days and seconds in day
        let mut days = timestamp.div_euclid(Self::SECONDS_PER_DAY);
        let day_seconds = timestamp.rem_euclid(Self::SECONDS_PER_DAY);

        let hour = (day_seconds / Self::SECONDS_PER_HOUR) as u8;
        let minute = ((day_seconds % Self::SECONDS_PER_HOUR) / Self::SECONDS_PER_MINUTE) as u8;
        let second = (day_seconds % Self::SECONDS_PER_MINUTE) as u8;

        // Compute year
        let mut year: i32 = 1970;

        if days >= 0 {
            loop {
                let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
                if days < days_in_year {
                    break;
                }
                days -= days_in_year;
                year += 1;
            }
        } else {
            loop {
                year -= 1;
                let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
                days += days_in_year;
                if days >= 0 {
                    break;
                }
            }
        }

        // Compute month
        let mut month: u8 = 1;
        loop {
            let dim = Self::days_in_month(month, year) as i64;
            if days < dim {
                break;
            }
            days -= dim;
            month += 1;
        }

        let day = (days + 1) as u8;

        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            timezone: 0,
            daylight_saving_time: false,
        })
    }

    /// Converts date/time to Unix timestamp (UTC)
    /// Output: i64 (Unix timestamp)
    pub fn to_timestamp(&self) -> i64 {
        let mut total_seconds: i64 = 0;

        // Add seconds, minutes, hours
        total_seconds += self.second as i64;
        total_seconds += (self.minute as i64) * 60;
        total_seconds += (self.hour as i64) * 3600;

        // Add days from start of year to current day
        let mut days_in_current_year: i64 = 0;
        for m in 1..self.month {
            days_in_current_year += Self::days_in_month(m, self.year) as i64;
        }
        days_in_current_year += (self.day - 1) as i64;
        total_seconds += days_in_current_year * Self::SECONDS_PER_DAY;

        // Add days from all complete years since 1970
        let start_year = 1970;
        if self.year >= start_year {
            for y in start_year..self.year {
                let days_in_year = if Self::is_leap_year(y) { 366 } else { 365 };
                total_seconds += days_in_year as i64 * Self::SECONDS_PER_DAY;
            }
        } else {
            for y in self.year..start_year {
                let days_in_year = if Self::is_leap_year(y) { 366 } else { 365 };
                total_seconds -= days_in_year as i64 * Self::SECONDS_PER_DAY;
            }
        }

        total_seconds
    }
}

impl Default for DateTime {
    fn default() -> Self {
        Self {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            timezone: 0,
            daylight_saving_time: false,
        }
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} (UTC{:+03}:{:02}){}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.timezone / 60,
            self.timezone % 60,
            if self.daylight_saving_time {
                " DST"
            } else {
                ""
            }
        )
    }
}

impl Debug for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Time {{ year: {}, month: {}, day: {}, hour: {}, minute: {}, second: {}, timezone: {}, daylight_saving_time: {} }}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.timezone,
            self.daylight_saving_time
        )
    }
}
