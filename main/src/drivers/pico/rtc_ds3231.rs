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

use osal_rs::println;
use osal_rs::utils::Result;

use crate::drivers::{i2c::{I2C, I2CFn}, platform::{I2C_BAUDRATE, I2C0_INSTANCE}, rtc::RTCFn};

pub const RTC_DS3231_I2C_ADDRESS: u8 = 0x68;

mod registers {
    pub(super) const SECONDS: u8 =  0x00;
    pub(super) const MINUTES: u8 =  0x01;
    pub(super) const HOURS: u8 =  0x02;
    pub(super) const DAY: u8 =  0x03;
    pub(super) const DATE: u8 =  0x04;
    pub(super) const MONTH_CENTURY: u8 =  0x05;
    pub(super) const YEAR: u8 =  0x06;
    pub(super) const CONTROL: u8 =  0x0E; 
    pub(super) const CONTROL_STATUS: u8 = 0x0F;
}

pub const RTC_FN: RTCFn = RTCFn {
    init,
    synch,
    set_rtc,
    get_rtc
};


fn init(i2c: &mut I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Result<()> {

    i2c.set_address(RTC_DS3231_I2C_ADDRESS);

    let data = [registers::SECONDS];
    let mut buffer = [0u8; 16];
    
    let (w,r) = i2c.write_and_read(&data, &mut buffer);

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