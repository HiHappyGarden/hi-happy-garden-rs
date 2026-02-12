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

use alloc::format;
use alloc::string::String;
use core::ffi::c_void;

use osal_rs::utils::{Error, OsalRsBool, Result};

use crate::drivers::i2c::I2CFn;
use crate::drivers::pico::ffi::{gpio_function_type, hhg_gpio_pull_up, hhg_gpio_set_function, hhg_i2c_init, hhg_i2c_init_pins_with_func, hhg_i2c_instance, hhg_i2c_read_blocking, hhg_i2c_write_blocking};
use crate::drivers::plt::ffi::pico_error_codes::PICO_OK;

pub const I2C0_INSTANCE: u8 = 0;
pub const I2C1_INSTANCE: u8 = 1;
pub const I2C_BAUDRATE: u32 = 100_000;

pub static I2C_FN: I2CFn = I2CFn {
    init,
    write,
    read,
    write_and_read,
    drop
};

fn init(i2c_instance: u8, baudrate: u32) -> Result<*mut c_void> {
    let i2c = unsafe { hhg_i2c_instance(i2c_instance) }; 
    if i2c.is_null() {
        return Err(Error::NullPtr);
    }

    unsafe {

        if I2C0_INSTANCE == i2c_instance {
            hhg_gpio_set_function(16, gpio_function_type::GPIO_FUNC_I2C as u32);
            hhg_gpio_set_function(17, gpio_function_type::GPIO_FUNC_I2C as u32);
            hhg_gpio_pull_up(16);
            hhg_gpio_pull_up(17);
        } else if I2C1_INSTANCE == i2c_instance {
            hhg_gpio_set_function(2, gpio_function_type::GPIO_FUNC_I2C as u32);
            hhg_gpio_set_function(3, gpio_function_type::GPIO_FUNC_I2C as u32);
            hhg_gpio_pull_up(2);
            hhg_gpio_pull_up(3);
        } else {
            return Err(Error::UnhandledOwned(format!("Invalid I2C instance: {}", i2c_instance)));
        }



        let res = hhg_i2c_init(i2c, baudrate);
        if res != baudrate {
            return Err(Error::Unhandled("I2C init failed negotiated baudrate"));
        }

        hhg_i2c_init_pins_with_func();
    }

    Ok(i2c)
}

fn write(instance: *mut c_void, address: u8, data: &[u8]) -> i32 {
    unsafe {
        hhg_i2c_write_blocking(instance, address, data.as_ptr(), data.len(), false)
    }
}
fn read(instance: *mut c_void, address: u8, buffer: &mut [u8]) -> i32 {
    unsafe {
        hhg_i2c_read_blocking(instance, address, buffer.as_mut_ptr(), buffer.len(), true)
    }
}

fn write_and_read(instance: *mut c_void, address: u8, data: &[u8], buffer: &mut [u8]) -> (i32, i32) {

    let mut ret = (0, 0);

    unsafe {
        if data.len() > 0 {
            ret.0 = hhg_i2c_write_blocking(instance, address, data.as_ptr(), data.len(), true);
            if ret.0 != PICO_OK as i32 {
                return ret;
            }
        }

        if buffer.len() > 0 {
            ret.1 = hhg_i2c_read_blocking(instance, address, buffer.as_mut_ptr(), buffer.len(), true);
            if ret.1 != PICO_OK as i32 {
                return ret;
            }
        }
    }
    ret
}

fn drop(instance: *mut c_void) {
    // No need to drop anything for now, but we can implement this if needed in the future
}