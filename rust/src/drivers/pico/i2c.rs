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

use osal_rs::utils::{Error, OsalRsBool, Result};

use crate::drivers::{i2c::I2CFn, pico::ffi::{hhg_i2c_init, hhg_i2c_init_pins_with_func, hhg_i2c_instance, hhg_i2c_read_blocking, hhg_i2c_write_blocking}};

pub const I2C_SH1106_INSTANCE: u8 = 0;
pub const I2C_SH1106_BAUDRATE: u32 = 100_000;

pub static I2C_FN: I2CFn = I2CFn {
    init: init,
    write: write,
    read: read,
    write_and_read: write_and_read,
};

fn init(i2c_instance: u8, baudrate: u32) -> Result<*mut c_void> {
    let i2c = unsafe { hhg_i2c_instance(i2c_instance) }; 
    if i2c.is_null() {
        return Err(Error::NullPtr);
    }

    unsafe {
        let res = hhg_i2c_init(i2c, baudrate);
        if res == 0 {
            return Err(Error::ReturnWithCode(res as i32));
        }

        hhg_i2c_init_pins_with_func();
    }

    Ok(i2c)
}

fn write(instance: *mut c_void, address: u8, data: &[u8]) -> OsalRsBool {
    unsafe {
        hhg_i2c_write_blocking(instance, address, data.as_ptr(), data.len(), false)
    }
    OsalRsBool::True
}
fn read(instance: *mut c_void, address: u8, buffer: &mut [u8]) -> OsalRsBool {
    unsafe {
        hhg_i2c_write_blocking(instance, address, buffer.as_mut_ptr(), buffer.len(), false)
    }
    OsalRsBool::True
}

fn write_and_read(instance: *mut c_void, address: u8, data: &[u8], buffer: &mut [u8]) -> OsalRsBool {
    unsafe {

        if data.len() > 0 {
            hhg_i2c_write_blocking(instance, address, data.as_ptr(), data.len(), false);
        }

        if buffer.len() > 0 {
            hhg_i2c_read_blocking(instance, address, buffer.as_mut_ptr(), buffer.len(), true);
        }
    }
    OsalRsBool::True
}