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

use core::ffi::c_void;
use core::ptr::null_mut;

use osal_rs::log_info;
use osal_rs::utils::{OsalRsBool, Result};

use crate::traits::state::Initializable;
use crate::drivers::platform::I2C_FN;

const APP_TAG: &str = "I2C";

pub struct I2CFn {
    pub init: fn(u8, u32) -> Result<*mut c_void>, //i2c_instance, baudrate
    pub write: fn(*mut c_void, u8, data: &[u8]) -> i32, //instance, address, data
    pub read: fn(*mut c_void, u8, buffer: &mut [u8]) -> i32, //instance, address, buffer
    pub write_and_read: fn(*mut c_void, u8, data: &[u8], buffer: &mut [u8]) -> OsalRsBool, //instance, address, data, buffer
}


 pub struct I2C<const ADDRESS: u8>  {
    functions: &'static I2CFn,
    i2c_instance: u8,
    instance: *mut c_void,
    baudrate: u32,
 }

 impl<const ADDRESS: u8> Initializable for I2C<ADDRESS> {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init i2c address: 0x{:02X}", ADDRESS);

        self.instance = (self.functions.init)(1, self.baudrate)?;

        Ok(())
    }
 }



 impl<const ADDRESS: u8> I2C<ADDRESS> {
    pub fn new(i2c_instance: u8, baudrate: u32) -> Self {
        Self{
            functions: &I2C_FN,
            i2c_instance,
            instance: null_mut(),
            baudrate,
        }
    }

    pub fn write(&self, data: &[u8]) -> i32 {
        (self.functions.write)(self.instance, ADDRESS, data)
    }

    pub fn read(&self, buffer: &mut [u8]) -> i32 {
        (self.functions.read)(self.instance, ADDRESS, buffer)
    }

    pub fn write_and_read(&self, data: &[u8], buffer: &mut [u8]) -> OsalRsBool {
        (self.functions.write_and_read)(self.instance, ADDRESS, data, buffer)
    }
}