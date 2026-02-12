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

use alloc::sync;
use osal_rs::log_info;
use osal_rs::utils::Result;

use crate::drivers::i2c::{I2C, I2CFn};
use crate::drivers::pico::rtc_ds3231::RTC_FN;
use crate::drivers::platform::{I2C0_INSTANCE, I2C_BAUDRATE};
use crate::traits::state::Initializable;

const APP_TAG: &str = "RTC";

const MINIMUM_DATE: i64 = 0;

#[derive(Clone, Debug)]
pub struct RTCFn {
    pub init: fn (&mut I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Result<()>,

    pub synch: fn (&I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>, timestamp: u64) -> Result<()>,

    pub set_rtc: fn (&I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>, timestamp: u64) -> Result<()>,

    pub get_rtc: fn (&I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Result<u64>, 
}

pub struct RTC (I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>);

impl Initializable for RTC {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init RTC");

        (RTC_FN.init)(&mut self.0)?;



        
        Ok(())
    }
}

impl RTC {

    #[inline]
    pub fn new(i2c: I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Self {
        Self (i2c)
    }

    #[inline]
    pub fn sync(&self, timestamp: u64) -> Result<()> {
        (RTC_FN.synch)(&self.0, timestamp)
    } 

    #[inline]
    pub fn set_rtc(&self, timestamp: u64) -> Result<()> {
        (RTC_FN.set_rtc)(&self.0, timestamp)
    }

    #[inline]
    pub fn get_rtc(&self) -> Result<u64> {
        (RTC_FN.get_rtc)(&self.0)
    }

    pub fn is_to_synch() -> bool {
        false
    }
}