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

use alloc::string::String;
use osal_rs::utils::{Result, Error};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    pub timezone: u16,
    pub daylight_saving_time: bool,
}

impl Time {

    /// New Input: year, month (1-12), day (1-31), hour (0-23), minute (0-59), second (0-59)
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8
    ) -> Result<Self> {

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

        Ok(Self { year, month, day, hour, minute, second, timezone: 0, daylight_saving_time: false })
    }

    /// Creates a Time from a Unix timestamp (UTC)
    /// Input: Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
    /// Output: Result<Time>
    pub fn new_from_unix_timestamp(timestamp: i64) -> Result<Self> {
        let mut remaining_seconds = timestamp;
        
        // Calculate year
        let start_year = 1970;
        let mut year = start_year;
        
        if remaining_seconds >= 0 {
            // Forward from 1970
            loop {
                let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
                let seconds_in_year = days_in_year * 24 * 3600;
                
                if remaining_seconds >= seconds_in_year {
                    remaining_seconds -= seconds_in_year;
                    year += 1;
                } else {
                    break;
                }
            }
        } else {
            // Backward from 1970
            loop {
                year -= 1;
                let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
                let seconds_in_year = days_in_year * 24 * 3600;
                
                remaining_seconds += seconds_in_year;
                if remaining_seconds >= 0 {
                    break;
                }
            }
        }
        
        // Calculate month and day
        let mut month = 1u8;
        while month <= 12 {
            let days_in_month = Self::days_in_month(month, year);
            let seconds_in_month = days_in_month as i64 * 24 * 3600;
            
            if remaining_seconds >= seconds_in_month {
                remaining_seconds -= seconds_in_month;
                month += 1;
            } else {
                break;
            }
        }
        
        // Calculate day
        let day = (remaining_seconds / (24 * 3600)) as u8 + 1;
        remaining_seconds %= 24 * 3600;
        
        // Calculate hour
        let hour = (remaining_seconds / 3600) as u8;
        remaining_seconds %= 3600;
        
        // Calculate minute
        let minute = (remaining_seconds / 60) as u8;
        
        // Calculate second
        let second = (remaining_seconds % 60) as u8;
        
        Self::new(year, month, day, hour, minute, second)
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
            2 => if Self::is_leap_year(year) { 29 } else { 28 },
            _ => 0, // invalid
        }
    }

    /// Converts date/time to Unix timestamp (UTC)
    /// Output: i64 (Unix timestamp) 
    pub fn to_unix_timestamp(&self) -> i64 {


        // Calculate total seconds from start of year
        let mut total_seconds = 0;

        // Add seconds, minutes, hours
        total_seconds += self.second as i64;
        total_seconds += (self.minute as i64) * 60;
        total_seconds += (self.hour as i64) * 3600;

        // Add days (from start of year to today)
        let mut days = 0;
        for m in 1..self.month {
            days += Self::days_in_month(m, self.year) as i64;
        }
        days += (self.day - 1) as i64; // -1 because day 1 is the first day of the month
        total_seconds += days * 24 * 3600;

        // Add years (from 1970 to today)
        let mut years = 0;
        let start_year = 1970;
        if self.year >= start_year {
            for y in start_year..self.year {
                years += if Self::is_leap_year(y) { 366 } else { 365 };
            }
        } else {
            for y in self.year..start_year {
                years -= if Self::is_leap_year(y) { 366 } else { 365 };
            }
        }
        total_seconds += years * 24 * 3600 * 365;
        // Add missing leap days (correction for leap years)
        if self.year > start_year {
            let mut leap_days = 0;
            for y in start_year..self.year {
                if Self::is_leap_year(y) {
                    leap_days += 1;
                }
            }
            total_seconds += leap_days * 24 * 3600;
        } else if self.year < start_year {
            let mut leap_days = 0;
            for y in self.year..start_year {
                if Self::is_leap_year(y) {
                    leap_days += 1;
                }
            }
            total_seconds -= leap_days * 24 * 3600;
        }

        total_seconds
    }
}