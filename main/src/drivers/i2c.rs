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
 
//#![allow(unused)]

use core::ffi::c_void;
use core::ptr::null_mut;

use osal_rs::log_info;
use osal_rs::utils::{OsalRsBool, Result};

use crate::traits::state::Initializable;
use crate::drivers::platform::I2C_FN;

const APP_TAG: &str = "I2C";

#[allow(unused)]
pub struct I2CFn {
    pub init: fn(u8, u32) -> Result<*mut c_void>, //i2c_instance, baudrate
    pub write: fn(*mut c_void, u8, data: &[u8]) -> i32, //instance, address, data
    pub read: fn(*mut c_void, u8, buffer: &mut [u8]) -> i32, //instance, address, buffer
    pub write_and_read: fn(*mut c_void, u8, data: &[u8], buffer: &mut [u8]) -> OsalRsBool, //instance, address, data, buffer
}

#[derive(Clone)]
 pub struct I2C<const INSTANCE: u8, const BAUDRATE: u32>  {
    instance: *mut c_void,
    address: u8,
 }

 unsafe impl<const INSTANCE: u8, const BAUDRATE: u32> Send for I2C<INSTANCE, BAUDRATE> {}
 unsafe impl<const INSTANCE: u8, const BAUDRATE: u32> Sync for I2C<INSTANCE, BAUDRATE> {}

 impl<const INSTANCE: u8, const BAUDRATE: u32> Initializable for I2C<INSTANCE, BAUDRATE> {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init i2c instance: {} baudrate: {}", INSTANCE, BAUDRATE);

        self.instance = (I2C_FN.init)(INSTANCE, BAUDRATE)?;

        Ok(())
    }
 }



 impl<const INSTANCE: u8, const BAUDRATE: u32> I2C<INSTANCE, BAUDRATE> {
    pub fn new(address: u8) -> Self {
        Self{
            instance: null_mut(),
            address,
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn set_address(&mut self, address: u8) {
        self.address = address;
    }

    #[inline]
    pub fn write(&self, data: &[u8]) -> i32 {
        (I2C_FN.write)(self.instance, self.address, data)
    }

    #[allow(unused)]
    #[inline]
    pub fn read(&self, buffer: &mut [u8]) -> i32 {
        (I2C_FN.read)(self.instance, self.address, buffer)
    }

    #[allow(unused)]
    #[inline]
    pub fn write_and_read(&self, data: &[u8], buffer: &mut [u8]) -> OsalRsBool {
        (I2C_FN.write_and_read)(self.instance, self.address, data, buffer)
    }
}