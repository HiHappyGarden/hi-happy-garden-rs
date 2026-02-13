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

use alloc::fmt::format;
use alloc::format;
use osal_rs::utils::{Error, Result};

use crate::drivers::i2c::{I2C, I2CFn};
use crate::drivers::pico::ffi::{hhg_powman_timer_get_ms, hhg_powman_timer_set_ms};
use crate::drivers::pico::rtc_ds3231::registers::*;
use crate::drivers::rtc::RTCFn;
use crate::drivers::platform::{I2C_BAUDRATE, I2C0_INSTANCE};
use crate::drivers::date_time::DateTime;

pub const RTC_DS3231_I2C_ADDRESS: u8 = 0x68;

const START_YEAR: i32 = 2000; // DS3231 counts years since 2000

mod registers {
    pub(super) const SECONDS: u8 =  0x00;
    pub(super) const MINUTES: u8 =  0x01;
    pub(super) const HOURS: u8 =  0x02;
    pub(super) const DAY_OF_MONTH: u8 =  0x04;
    pub(super) const MONTH_CENTURY: u8 =  0x05;
    pub(super) const YEAR: u8 =  0x06;
}

pub const RTC_FN: RTCFn = RTCFn {
    init,
    set_timestamp,
    get_timestamp,
    set_rtc_timestamp,
    get_rtc_timestamp
};


fn init(i2c: &mut I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Result<()> {

    i2c.set_address(RTC_DS3231_I2C_ADDRESS);

    Ok(())
}

#[inline]
fn set_timestamp(i2c: &I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>, timestamp: u64) {

    unsafe { hhg_powman_timer_set_ms(timestamp *  1_000) };
}

#[inline]
fn get_timestamp(i2c: &I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> u64 {
    unsafe {hhg_powman_timer_get_ms() / 1_000}
}




fn set_rtc_timestamp(i2c: &I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>, timestamp: i64) -> Result<()> {
    let time = DateTime::from_timestamp(timestamp)?;

    {
        let second = ((time.second / 10) << 4) | (time.second % 10);
        i2c.write(&[SECONDS, second])?;
    }
    
    {
        let minute = ((time.minute / 10) << 4) | (time.minute % 10);
        i2c.write(&[MINUTES, minute])?;
    }
    
    {
        let hour = {
        let lower = (time.hour % 10);
        let upper = match time.hour {
            10..=19 => 0x10, // 10-19
            20..=23 => 0x20, // 20-23
            _ => 0x00, // should never happen due to validation in Time::new
            };
            upper | lower
        };
        i2c.write(&[HOURS, hour])?;
    }
    

    {
        let day = ((time.day / 10) << 4) | (time.day % 10);
        i2c.write(&[DAY_OF_MONTH, day])?;
    }
    
    {
        let century = if time.year >= 2100 { 0x80 } else { 0x00 }; // bit 7 of MONTH register is century bit
        let month = {
            let lower = (time.month % 10);
            let upper = match time.month {
                10..=12 => 0x10, // 10-12
                _ => 0x00, // should never happen due to validation in Time::new
            };
            century | upper | lower
        };
        i2c.write(&[MONTH_CENTURY, month])?;
    }
    {
        let year = ((time.year - START_YEAR) / 10) << 4 | ((time.year - START_YEAR) % 10);
        let year = match year.try_into() {
            Ok(y) => y,
            Err(e) => return Err(Error::UnhandledOwned(format!("Year overflow u8 size error: {}", e))),
        };
        i2c.write(&[YEAR, year])?;
    }
    
    Ok(())
}

fn get_rtc_timestamp (i2c: &I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Result<i64> {

    let second = {
        let data = [SECONDS];
        let mut buffer = [0u8; 7];
        i2c.write_and_read(&data, &mut buffer)?;

        let upper = buffer[0] & 0xF0;
        let lower = buffer[0] & 0x0F;
        (upper >> 4) * 10 + lower
    };

    let minute = {
        let data = [MINUTES];
        let mut buffer = [0u8; 7];
        i2c.write_and_read(&data, &mut buffer)?;

        let upper = buffer[0] & 0xF0;
        let lower = buffer[0] & 0x0F;
        (upper >> 4) * 10 + lower
    };

    let hour = {
        let data = [HOURS];
        let mut buffer = [0u8; 7];
        i2c.write_and_read(&data, &mut buffer)?;

        let mode24h = buffer[0] & 0x40 == 0; // bit 6 is 0 for 24h mode, 1 for 12h mode
        if mode24h {
            let hour20 = (buffer[0] & 0x20) >> 5;
            let hour10 = (buffer[0] & 0x10) >> 4;
            let lower = buffer[0] & 0x0F;
            if hour20 == 1 {
                20 + lower
            } else if hour10 == 1 {
                10 + lower
            } else {
                lower
            }
        } else {
            let upper = buffer[0] & 0x10;
            let lower = buffer[0] & 0x0F;
            let hour = (upper >> 4) * 10 + lower;
            if buffer[0] & 0x20 != 0 { // bit 5 is AM/PM in 12h mode
                (hour % 12) + 12 // PM
            } else {
                hour % 12 // AM
            }
        }
    };


    let day = {
        let data = [DAY_OF_MONTH];
        let mut buffer = [0u8; 7];
        i2c.write_and_read(&data, &mut buffer)?;

        let upper = buffer[0] & 0x30;
        let lower = buffer[0] & 0x0F;
        (upper >> 4) * 10 + lower
    };

    let month = {
        let data = [MONTH_CENTURY];
        let mut buffer = [0u8; 7];
        i2c.write_and_read(&data, &mut buffer)?;

        let upper = buffer[0] & 0x10;
        let lower = buffer[0] & 0x0F;
        ( ((upper >> 4) * 10 + lower), (buffer[0] & 0x80) >> 7 )
    };

    let year = {
        let data = [YEAR];
        let mut buffer = [0u8; 7];
        i2c.write_and_read(&data, &mut buffer)?;

        let upper = buffer[0] & 0xF0;
        let lower = buffer[0] & 0x0F;
        (upper >> 4) * 10 + lower
    };
    let mut year = (month.1 as i32 * 100) + year as i32;
    year += START_YEAR;

    let month = month.0;

    let mut time = DateTime::new(year, month, day, hour, minute, second)?;

    Ok(time.to_timestamp())
}