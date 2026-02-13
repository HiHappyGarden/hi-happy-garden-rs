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

use core::time;

use alloc::sync;
use osal_rs::log_info;
use osal_rs::utils::{Error, Result};

use crate::drivers::i2c::{I2C, I2CFn};
use crate::drivers::pico::rtc_ds3231::RTC_FN;
use crate::drivers::platform::{I2C0_INSTANCE, I2C_BAUDRATE};
use crate::drivers::date_time::DateTime;
use crate::traits::state::Initializable;

const APP_TAG: &str = "RTC";

#[derive(Clone, Debug)]
pub struct RTCFn {
    pub init: fn (&mut I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Result<()>,

    pub set_timestamp: fn (&I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>, timestamp: u64),

    #[allow(unused)]
    pub get_timestamp: fn (&I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> u64,

    #[allow(unused)]
    pub set_rtc_timestamp: fn (&I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>, timestamp: i64) -> Result<()>,

    pub get_rtc_timestamp: fn (&I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) -> Result<i64>, 
}

pub struct RTC (Option<I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>>);

impl Initializable for RTC {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init RTC");

        if self.0.is_none() {
            return Err(Error::NullPtr);
        }
        (RTC_FN.init)(&mut self.0.as_mut().unwrap())?;
        
        Ok(())
    }
}

impl RTC {
    pub const MINIMUM_DATE: i64 = 1_577_836_800; // 2020-01-01T00:00:00Z

    #[inline]
    pub const fn new() -> Self {
        Self (None)
    }

    #[inline]
    pub fn set_i2c(&mut self, i2c: I2C<{I2C0_INSTANCE}, {I2C_BAUDRATE}>) {
        self.0 = Some(i2c);
    }

    #[inline]
    pub fn set_timestamp(&self, timestamp: u64) -> Result<()> {
        if self.0.is_none() {
            return Err(Error::NullPtr);
        }
        (RTC_FN.set_timestamp)(&self.0.as_ref().unwrap(), timestamp);
        Ok(())
    } 

    #[allow(unused)]
    #[inline]
    pub fn get_timestamp(&self) -> Result<u64> {
        if self.0.is_none() {
            return Err(Error::NullPtr);
        }
        Ok((RTC_FN.get_timestamp)(&self.0.as_ref().unwrap()))
    }

    #[allow(unused)]
    #[inline]
    pub fn set_rtc_timestamp(&self, timestamp: i64) -> Result<()> {
        if self.0.is_none() {
            return Err(Error::NullPtr);
        }
        (RTC_FN.set_rtc_timestamp)(&self.0.as_ref().unwrap(), timestamp)
    }

    #[inline]
    pub fn get_rtc_timestamp(&self) -> Result<i64> {
        if self.0.is_none() {
            return Err(Error::NullPtr);
        }
        (RTC_FN.get_rtc_timestamp)(&self.0.as_ref().unwrap())
    }

    pub fn is_to_synch(&self) -> bool {
        if self.0.is_none() {
            return true; // if we don't have an I2C instance, we assume we need to sync
        }

        let ret = self.get_rtc_timestamp();
        if ret.is_err() {
            return true; // if we can't read the RTC timestamp, we assume we need to sync
        }

        if ret.unwrap() > Self::MINIMUM_DATE {
            true
        } else {
            false
        }
    }

}