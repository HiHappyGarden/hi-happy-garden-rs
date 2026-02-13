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
use core::ffi::c_void;

use osal_rs::utils::{Error, Result};

use crate::drivers::i2c::I2CFn;
use crate::drivers::pico::ffi::{gpio_function_type, hhg_gpio_pull_up, hhg_gpio_set_function, hhg_i2c_deinit, hhg_i2c_init, hhg_i2c0_init_pins_with_func, hhg_i2c1_init_pins_with_func, hhg_i2c_instance, hhg_i2c_read_blocking, hhg_i2c_write_blocking};
use crate::drivers::plt::ffi::pico_error_codes::{PICO_OK, PICO_ERROR_TIMEOUT, PICO_ERROR_GENERIC};

pub const I2C0_INSTANCE: u8 = 0;
pub const I2C0_PIN_SDA: u32 = 16;
pub const I2C0_PIN_SCL: u32 = 17;


pub const I2C1_INSTANCE: u8 = 1;
pub const I2C1_PIN_SDA: u32 = 2;
pub const I2C1_PIN_SCL: u32 = 3;

pub const I2C_BAUDRATE: u32 = 100_000;

pub static I2C_FN: I2CFn = I2CFn {
    init,
    write,
    read,
    write_and_read,
    scan_i2c,
    drop
};

/// Scan I2C bus for devices (utility function for debugging)
pub fn scan_i2c(instance: *mut c_void) -> Result<alloc::vec::Vec<u8>> {
    let mut devices = alloc::vec::Vec::new();
    
    for addr in 0x08..=0x77 {
        let mut buf = [0u8; 1];
        unsafe {
            let res = hhg_i2c_read_blocking(instance, addr, buf.as_mut_ptr(), 1, false);
            if res == PICO_OK as i32 || res >= 0 {
                devices.push(addr);
            }
        }
    }
    
    Ok(devices)
}

fn init(i2c_instance: u8, baudrate: u32) -> Result<*mut c_void> {
    let i2c = unsafe { hhg_i2c_instance(i2c_instance) }; 
    if i2c.is_null() {
        return Err(Error::NullPtr);
    }

    unsafe {

        if I2C0_INSTANCE == i2c_instance {
            // Configure GPIO pins for I2C function
            hhg_gpio_set_function(I2C0_PIN_SDA, gpio_function_type::GPIO_FUNC_I2C as u32);
            hhg_gpio_set_function(I2C0_PIN_SCL, gpio_function_type::GPIO_FUNC_I2C as u32);
            
            // Enable internal pull-ups (may not be sufficient - check hardware)
            hhg_gpio_pull_up(I2C0_PIN_SDA);
            hhg_gpio_pull_up(I2C0_PIN_SCL);

            // Initialize I2C peripheral
            let res = hhg_i2c_init(i2c, baudrate);
            if res != baudrate {
                return Err(Error::UnhandledOwned(format!(
                    "I2C init failed: requested {} Hz, got {} Hz", 
                    baudrate, res
                )));
            }

            hhg_i2c0_init_pins_with_func();

        } else if I2C1_INSTANCE == i2c_instance {
            // Configure GPIO pins for I2C function
            hhg_gpio_set_function(I2C1_PIN_SDA, gpio_function_type::GPIO_FUNC_I2C as u32);
            hhg_gpio_set_function(I2C1_PIN_SCL, gpio_function_type::GPIO_FUNC_I2C as u32);
            
            // Enable internal pull-ups (may not be sufficient - check hardware)
            hhg_gpio_pull_up(I2C1_PIN_SDA);
            hhg_gpio_pull_up(I2C1_PIN_SCL);

            // Initialize I2C peripheral
            let res = hhg_i2c_init(i2c, baudrate);
            if res != baudrate {
                return Err(Error::UnhandledOwned(format!(
                    "I2C init failed: requested {} Hz, got {} Hz", 
                    baudrate, res
                )));
            }

            hhg_i2c1_init_pins_with_func();

        } else {
            return Err(Error::UnhandledOwned(format!("Invalid I2C instance: {}", i2c_instance)));
        }


    }

    Ok(i2c)
}

fn write(instance: *mut c_void, address: u8, data: &[u8]) -> Result<()> {
    unsafe {
        let res = hhg_i2c_write_blocking(instance, address, data.as_ptr(), data.len(), false);
        // i2c_write_blocking returns number of bytes written (positive) on success,
        // or negative error code (PICO_ERROR_*) on failure
        if res < 0 {
            // Provide more detailed error information
            if res == PICO_ERROR_TIMEOUT as i32 {
                Err(Error::UnhandledOwned(format!(
                    "I2C write timeout (addr: 0x{:02X}) - Check: 1) Device connected? 2) Correct address? 3) External pull-ups (4.7kΩ) installed?",
                    address
                )))
            } else if res == PICO_ERROR_GENERIC as i32 {
                Err(Error::UnhandledOwned(format!(
                    "I2C write generic error (addr: 0x{:02X}) - Device not responding",
                    address
                )))
            } else {
                Err(Error::ReturnWithCode(res))
            }
        } else {
            Ok(())
        }
    }
}

fn read(instance: *mut c_void, address: u8, buffer: &mut [u8]) -> Result<()> {
    unsafe {
        let res = hhg_i2c_read_blocking(instance, address, buffer.as_mut_ptr(), buffer.len(), true);
        // i2c_read_blocking returns number of bytes read (positive) on success,
        // or negative error code (PICO_ERROR_*) on failure
        if res < 0 {
            if res == PICO_ERROR_TIMEOUT as i32 {
                Err(Error::UnhandledOwned(format!(
                    "I2C read timeout (addr: 0x{:02X}) - Check device connection and address",
                    address
                )))
            } else {
                Err(Error::ReturnWithCode(res))
            }
        } else {
            Ok(())
        }
    }
}

fn write_and_read(instance: *mut c_void, address: u8, data: &[u8], buffer: &mut [u8]) -> (Result<()>, Result<()>) {

    let mut ret = (0, 0);

    unsafe {
        if data.len() > 0 {
            ret.0 = hhg_i2c_write_blocking(instance, address, data.as_ptr(), data.len(), true);
            // Check for negative error codes
            if ret.0 < 0 {
                return (Err(Error::ReturnWithCode(ret.0)), Ok(()));
            }
        }

        if buffer.len() > 0 {
            ret.1 = hhg_i2c_read_blocking(instance, address, buffer.as_mut_ptr(), buffer.len(), true);
            // Check for negative error codes
            if ret.1 < 0 {
                return (Ok(()), Err(Error::ReturnWithCode(ret.1)));
            }
        }
    }
    (Ok(()), Ok(()))
}

fn drop(instance: *mut c_void) {
    unsafe {
        hhg_i2c_deinit(instance);
    }
}