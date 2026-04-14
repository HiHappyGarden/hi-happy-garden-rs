/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
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

#![allow(unused)]

use core::fmt::{Debug, Display};

use alloc::string::String;
use osal_rs::utils::{Error, Result};

use crate::drivers::rtc::RTC;

static mut TIMEZONE: i16 = 0; // in minutes, e.g. +120 for UTC+2, -60 for UTC-1
static mut DAYLIGHT_SAVING_TIME_ENABLED: bool = false; // true if daylight saving time is in effect
static mut START_MONTH: u8 = 0;
static mut START_DAY: u8 = 0;
static mut START_HOUR: u8 = 0;
static mut END_MONTH: u8 = 0;
static mut END_DAY: u8 = 0;
static mut END_HOUR: u8 = 0;


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    pub year: i32,     //starting from 1970, can be negative for dates before 1970
    pub month: u8,     // 1-12
    pub mday: u8,      // 1-31
    pub wday: u8,      // 0-6 (0=Sunday, 1=Monday, ..., 6=Saturday)
    pub hour: u8,      // 0-23
    pub minute: u8,    // 0-59
    pub second: u8,     // 0-59
    pub millis: u16,      // 0-999 (not used in timestamp conversion, but can be useful for display or RTC purposes)
    is_apply_timezone: bool,
    is_apply_daylight_saving_time: bool
}

impl DateTime {
    pub const SECONDS_PER_MINUTE: i64 = 60;
    pub const SECONDS_PER_HOUR: i64 = 3_600;
    pub const SECONDS_PER_DAY: i64 = 86_400;

    pub fn set_daylight_saving_time(enabled: bool, start_month: u8, start_day: u8, start_hour: u8, end_month: u8, end_day: u8, end_hour: u8) {
        unsafe {
            DAYLIGHT_SAVING_TIME_ENABLED = enabled;
            START_MONTH = start_month;
            START_DAY = start_day;
            START_HOUR = start_hour;
            END_MONTH = end_month;
            END_DAY = end_day;
            END_HOUR = end_hour;
        }
    }

    pub fn set_timezone(offset_minutes: i16) {
        unsafe {
            TIMEZONE = offset_minutes;
        }
    }

    /// New Input: year, month (1-12), wday (0-6), mday (1-31), hour (0-23), minute (0-59), second (0-59)
    pub fn new(year: i32, month: u8, wday: u8, mday: u8, hour: u8, minute: u8, second: u8) -> Result<Self> {
        // Validate input
        if month < 1 || month > 12 {
            return Err(Error::Unhandled("Invalid month"));
        }
        if mday < 1 || mday > Self::days_in_month(month, year) {
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
            wday,
            mday,
            hour,
            minute,
            second,
            millis: 0,
            is_apply_timezone: false,
            is_apply_daylight_saving_time: false
            
        })
    }

    /// New Input: year, month (1-12), day (1-31), hour (0-23), minute (0-59), second (0-59)
    pub fn new_date(year: i32, month: u8, mday: u8) -> Result<Self> {
        // Validate input
        if month < 1 || month > 12 {
            return Err(Error::Unhandled("Invalid month"));
        }
        if mday < 1 || mday > Self::days_in_month(month, year) {
            return Err(Error::Unhandled("Invalid day for given month/year"));
        }

        Ok(Self {
            year,
            month,
            wday: 0, // wday can be calculated if needed using the to_timestamp_locale() method
            mday,
            hour: 0,
            minute: 0,
            second: 0,
            millis: 0,
            is_apply_timezone: false,
            is_apply_daylight_saving_time: false
            
        })
    }


    /// New Input: hour (0-23), minute (0-59), second (0-59)
    pub fn new_time(hour: u8, minute: u8, second: u8) -> Result<Self> {
        // Validate input
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
            year: 0,
            month: 0,
            wday: 0,
            mday: 0,
            hour,
            minute,
            second,
            millis: 0,
            is_apply_timezone: false,
            is_apply_daylight_saving_time: false
        })
    }


    /// Returns true if the year is a leap year
    #[inline]
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Returns the number of days in a given month (accounting for leap year)
    pub fn days_in_month(month: u8, year: i32) -> u8 {
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

    /// Helper function to find the last occurrence of a specific weekday in a month
    /// weekday: 0=Sunday, 1=Monday, ..., 6=Saturday
    fn find_last_weekday_of_month(year: i32, month: u8, weekday: u8) -> u8 {
        let days_in_month = Self::days_in_month(month, year);
        
        // Start from the last day of the month and work backwards
        for day in (1..=days_in_month).rev() {
            // Calculate wday for this date
            let ts = Self::new(year, month, 0, day, 0, 0, 0)
                .map(|dt| dt.to_timestamp_locale())
                .unwrap_or(0);
            let calculated_wday = ((ts / Self::SECONDS_PER_DAY + 4) % 7) as u8;
            
            if calculated_wday == weekday {
                return day;
            }
        }
        
        // Fallback (should never happen)
        days_in_month
    }

    /// Checks if the current date/time is within the daylight saving time period based on the configured start and end dates/times
    /// START_DAY and END_DAY should be in range 1-31 (not 0-based)
    /// Special value: 0xFF means "last Sunday of the month" (for EU DST rules)
    /// If START_DAY or END_DAY is greater than the number of days in the month, it will use the last day of that month
    fn is_daylight_saving_time(&self) -> bool {
        if !unsafe {DAYLIGHT_SAVING_TIME_ENABLED} {
            return false;
        }

        let start_month = unsafe {START_MONTH};
        let start_day = unsafe {START_DAY};
        let start_hour = unsafe {START_HOUR};
        let end_month = unsafe {END_MONTH};
        let end_day = unsafe {END_DAY};
        let end_hour = unsafe {END_HOUR};

        // Calculate DST start date for current year
        let start_mday = if start_day == 0xFF {
            // Special case: last Sunday of the month
            Self::find_last_weekday_of_month(self.year, start_month, 0)
        } else {
            start_day.min(Self::days_in_month(start_month, self.year))
        };

        // Calculate DST end date for current year
        let end_mday = if end_day == 0xFF {
            // Special case: last Sunday of the month
            Self::find_last_weekday_of_month(self.year, end_month, 0)
        } else {
            end_day.min(Self::days_in_month(end_month, self.year))
        };

        // Compare dates directly instead of using timestamps
        // This avoids issues with timezone/DST already applied to self
        
        // Create comparison values as (month, day, hour)
        let now = (self.month, self.mday, self.hour);
        let start = (start_month, start_mday, start_hour);
        let end = (end_month, end_mday, end_hour);

        // Handle both Northern and Southern hemisphere cases
        if start < end {
            // Northern hemisphere: DST starts in spring, ends in autumn
            now >= start && now < end
        } else {
            // Southern hemisphere: DST starts in autumn (year Y), ends in spring (year Y+1)
            now >= start || now < end
        }
    }


    #[inline]
    pub fn from_timestamp(timestamp: i64) -> Result<Self> {
        Self::from_timestamp_locale(timestamp, false)
    }

    /// Internal: Creates a DateTime from timestamp without any timezone/DST conversion
    fn from_timestamp_raw(timestamp: i64) -> Result<Self> {
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

        let mday = (days + 1) as u8;
        let wday = ((timestamp / Self::SECONDS_PER_DAY + 4).rem_euclid(7)) as u8;

        Ok(Self {
            year,
            month,
            wday,
            mday,
            hour,
            minute,
            second,
            millis: 0,
            is_apply_timezone: false,
            is_apply_daylight_saving_time: false,
        })
    }

    /// Creates a Time from a Unix timestamp (UTC)
    /// Input: Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
    /// Output: Result<Time>
    pub fn from_timestamp_locale(timestamp: i64, locale: bool) -> Result<Self> {
        let mut adjusted_timestamp = timestamp;
        let mut is_apply_timezone = false;
        let mut is_apply_daylight_saving_time = false;

        if locale {
            // First apply timezone offset (convert from UTC to local time)
            if unsafe { TIMEZONE != 0 } {
                let tz_offset_seconds = unsafe { TIMEZONE as i64 } * Self::SECONDS_PER_MINUTE;
                adjusted_timestamp += tz_offset_seconds;
                is_apply_timezone = true;
            }

            // Check if DST is active by verifying on the UTC timestamp
            // We need to check the LOCAL date (after timezone) to see if DST applies
            let temp = Self::from_timestamp_raw(adjusted_timestamp)?;
            if temp.is_daylight_saving_time() {
                let dst_offset_seconds = Self::SECONDS_PER_HOUR; // 1 hour DST offset
                adjusted_timestamp += dst_offset_seconds;
                is_apply_daylight_saving_time = true;
            }
        }

        // Use the raw conversion and update flags
        let mut result = Self::from_timestamp_raw(adjusted_timestamp)?;
        result.is_apply_timezone = is_apply_timezone;
        result.is_apply_daylight_saving_time = is_apply_daylight_saving_time;
        
        Ok(result)
    }


    pub fn to_timestamp(&self) -> i64 {
        let mut sum = self.to_timestamp_locale();

        if self.is_apply_daylight_saving_time {
            let dst_offset_seconds = Self::SECONDS_PER_HOUR; // we assume a fixed 1 hour DST offset, this can be made configurable if needed
            sum -= dst_offset_seconds;
        }

        if self.is_apply_timezone {
            let tz_offset_seconds = unsafe { TIMEZONE as i64 } * Self::SECONDS_PER_MINUTE;
            sum -= tz_offset_seconds;
        }
        
        sum
    }

    /// Converts date/time to Unix timestamp (UTC)
    /// Output: i64 (Unix timestamp)
    pub fn to_timestamp_locale(&self) -> i64 {
        let mut total_seconds: i64 = 0;

        // Add seconds, minutes, hours
        total_seconds += self.second as i64;
        total_seconds += (self.minute as i64) * Self::SECONDS_PER_MINUTE;
        total_seconds += (self.hour as i64) * Self::SECONDS_PER_HOUR;

        // Add days from start of year to current day
        let mut days_in_current_year: i64 = 0;
        for m in 1..self.month {
            days_in_current_year += Self::days_in_month(m, self.year) as i64;
        }
        days_in_current_year += (self.mday - 1) as i64;
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

    #[inline]
    pub fn is_apply_timezone(&self) -> bool {
        self.is_apply_timezone
    }

    #[inline]
    pub fn is_apply_daylight_saving_time(&self) -> bool {
        self.is_apply_daylight_saving_time
    }

    pub fn is_valid(&self) -> bool {
        *self != Self::default()
    }

}

impl Default for DateTime {
    fn default() -> Self {
        Self {
            year: 1970,
            month: 1,
            wday: 4, // 1970-01-01 was a Thursday
            mday: 1,
            hour: 0,
            minute: 0,
            second: 0,
            millis: 0,
            is_apply_timezone: false,
            is_apply_daylight_saving_time: false,
        }
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_apply_timezone || self.is_apply_daylight_saving_time {
            let tz_offset = unsafe { TIMEZONE };
            let tz_hours = tz_offset / 60;
            let tz_minutes = (tz_offset % 60).abs(); // abs() to handle negative timezones correctly
            
            write!(
                f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} (UTC{:+03}:{:02}){}",
                self.year,
                self.month,
                self.mday,
                self.hour,
                self.minute,
                self.second,
                tz_hours,
                tz_minutes,
                if self.is_apply_daylight_saving_time {
                    " DST"
                } else {
                    ""
                }
            )
        } else {
            // UTC time without timezone offset
            write!(
                f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
                self.year,
                self.month,
                self.mday,
                self.hour,
                self.minute,
                self.second
            )
        }
    }
}

impl Debug for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Time {{ year: {}, month: {}, wday: {}, mday: {}, hour: {}, minute: {}, second: {}, millis: {}, timezone: {}, daylight_saving_time: {} }}",
            self.year,
            self.month,
            self.wday,
            self.mday,
            self.hour,
            self.minute,
            self.second,
            self.millis,
            self.is_apply_timezone,
            self.is_apply_daylight_saving_time
        )
    }
}