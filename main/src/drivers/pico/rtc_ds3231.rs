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

use osal_rs::utils::Result;

use crate::drivers::{i2c::{I2C, I2CFn}, platform::{I2C_BAUDRATE, I2C0_INSTANCE}, rtc::RTCFn};

mod registers {
    const SECONDS: u8 =  0x00;
    const MINUTES: u8 =  0x01;
    const HOURS: u8 =  0x02;
    const DAY: u8 =  0x03;
    const DATE: u8 =  0x04;
    const MONTH_CENTURY: u8 =  0x05;
    const YEAR: u8 =  0x06;
}

pub const RTC_FN: RTCFn = RTCFn {
    init,
    synch,
    set_rtc,
    get_rtc
};


fn init(i2c: &I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Result<()> {


    Ok(())
}

fn synch(i2c: &I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>, timestamp: u64) -> Result<()> {


    Ok(())
}

fn set_rtc(i2c: &I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>, timestamp: u64) -> Result<()> {
    Ok(())
}

fn get_rtc (i2c: &I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Result<u64> {
    Ok(0)
}