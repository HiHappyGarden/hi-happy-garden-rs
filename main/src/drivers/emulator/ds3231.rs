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

//! DS3231 battery backed RTC on I2C0, at register level.
//!
//! Powers up holding the host UTC time, as a board whose battery kept the
//! clock set, and from there counts seconds on its own. Registers are BCD,
//! the register pointer auto-increments and wraps like on the chip.
//!
//! The calendar math is written here from scratch on purpose: reusing the
//! firmware `DateTime` would hide its bugs instead of catching them.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::i2c::I2cDevice;

const SECONDS: usize = 0x00;
const MINUTES: usize = 0x01;
const HOURS: usize = 0x02;
const DAY_OF_WEEK: usize = 0x03;
const DATE: usize = 0x04;
const MONTH_CENTURY: usize = 0x05;
const YEAR: usize = 0x06;
const CONTROL: usize = 0x0E;

/// Registers 0x00 (seconds) to 0x12 (temperature LSB)
const REGISTER_COUNT: usize = 0x13;

const HOURS_12H: u8 = 0x40;
const HOURS_PM: u8 = 0x20;
const CENTURY: u8 = 0x80;

/// Power-on value of the control register: oscillator on, square wave off
const CONTROL_POWER_ON: u8 = 0x1C;

const SECONDS_PER_DAY: i64 = 86_400;

/// The chip counts years 2000-2199 with the century bit
const BASE_YEAR: i64 = 2000;

pub(super) struct Ds3231 {
    registers: [u8; REGISTER_COUNT],
    pointer: usize,
    /// Start of the second the time registers are holding
    tick: Instant,
}

impl Ds3231 {
    pub(super) const I2C_ADDRESS: u8 = 0x68;

    pub(super) fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or(0);

        let mut rtc = Self {
            registers: [0u8; REGISTER_COUNT],
            pointer: 0,
            tick: Instant::now(),
        };
        rtc.registers[CONTROL] = CONTROL_POWER_ON;
        rtc.set_time(now, weekday(now.div_euclid(SECONDS_PER_DAY)));
        rtc
    }

    /// Moves the time registers forward by the whole seconds elapsed.
    fn advance(&mut self) {
        let elapsed = self.tick.elapsed().as_secs();
        if elapsed == 0 {
            return;
        }
        self.tick += Duration::from_secs(elapsed);

        // Registers left inconsistent by the firmware stay as they are,
        // there is no meaningful time to count from
        let Some(before) = self.time() else {
            return;
        };
        let after = before + elapsed as i64;

        // The day of week is an independent counter on the chip: it moves
        // on at midnight from whatever the firmware wrote
        let days = after.div_euclid(SECONDS_PER_DAY) - before.div_euclid(SECONDS_PER_DAY);
        let wday = (i64::from(self.registers[DAY_OF_WEEK] & 0x07) - 1 + days).rem_euclid(7) as u8 + 1;

        self.set_time(after, wday);
    }

    fn set_time(&mut self, timestamp: i64, wday: u8) {
        let days = timestamp.div_euclid(SECONDS_PER_DAY);
        let second_of_day = timestamp.rem_euclid(SECONDS_PER_DAY);
        let (year, month, day) = civil_from_days(days);

        self.registers[SECONDS] = to_bcd((second_of_day % 60) as u8);
        self.registers[MINUTES] = to_bcd((second_of_day / 60 % 60) as u8);
        self.registers[HOURS] = to_bcd((second_of_day / 3_600) as u8);
        self.registers[DAY_OF_WEEK] = wday;
        self.registers[DATE] = to_bcd(day as u8);
        let century = if year >= BASE_YEAR + 100 { CENTURY } else { 0 };
        self.registers[MONTH_CENTURY] = century | to_bcd(month as u8);
        self.registers[YEAR] = to_bcd(((year - BASE_YEAR).rem_euclid(100)) as u8);
    }

    /// The time held by the registers, `None` if they do not form a valid date.
    fn time(&self) -> Option<i64> {
        let second = from_bcd(self.registers[SECONDS] & 0x7F);
        let minute = from_bcd(self.registers[MINUTES] & 0x7F);
        let hours = self.registers[HOURS];
        let hour = if hours & HOURS_12H != 0 {
            from_bcd(hours & 0x1F) % 12 + if hours & HOURS_PM != 0 { 12 } else { 0 }
        } else {
            from_bcd(hours & 0x3F)
        };
        let day = from_bcd(self.registers[DATE] & 0x3F);
        let month = from_bcd(self.registers[MONTH_CENTURY] & 0x1F);
        let century = if self.registers[MONTH_CENTURY] & CENTURY != 0 { 100 } else { 0 };
        let year = BASE_YEAR + century + i64::from(from_bcd(self.registers[YEAR]));

        if second > 59 || minute > 59 || hour > 23 || !(1..=12).contains(&month) {
            return None;
        }
        if day < 1 || i64::from(day) > days_in_month(year, i64::from(month)) {
            return None;
        }

        let days = days_from_civil(year, i64::from(month), i64::from(day));
        Some(days * SECONDS_PER_DAY + i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second))
    }
}

impl I2cDevice for Ds3231 {
    fn write(&mut self, data: &[u8]) {
        let Some((&pointer, values)) = data.split_first() else {
            return;
        };
        self.advance();
        self.pointer = pointer as usize % REGISTER_COUNT;

        for &value in values {
            self.registers[self.pointer] = value;
            // Writing the seconds resets the countdown chain of the chip
            if self.pointer == SECONDS {
                self.tick = Instant::now();
            }
            self.pointer = (self.pointer + 1) % REGISTER_COUNT;
        }
    }

    fn read(&mut self, buffer: &mut [u8]) {
        self.advance();
        for byte in buffer {
            *byte = self.registers[self.pointer];
            self.pointer = (self.pointer + 1) % REGISTER_COUNT;
        }
    }
}

fn to_bcd(value: u8) -> u8 {
    ((value / 10) << 4) | (value % 10)
}

fn from_bcd(value: u8) -> u8 {
    (value >> 4) * 10 + (value & 0x0F)
}

/// DS3231 day of week, 1 (Sunday) to 7, for days since 1970-01-01 (a Thursday)
fn weekday(days: i64) -> u8 {
    (days + 4).rem_euclid(7) as u8 + 1
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 of a proleptic Gregorian date (H. Hinnant's algorithm)
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Inverse of [`days_from_civil`]: (year, month, day)
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}
