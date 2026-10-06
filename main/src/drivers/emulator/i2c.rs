/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! The two I2C buses of the board with their devices.
//!
//! Twin of `src/pico/hhg-i2c-wrapper.c`: I2C0 carries the DS3231 RTC, I2C1
//! the SH1106 display. Every other address NACKs, which the pico-sdk
//! reports as `PICO_ERROR_GENERIC`.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::{c_uint, c_void};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use std::sync::{LazyLock, Mutex};

use crate::drivers::pico::ffi::pico_error_codes::{PICO_ERROR_GENERIC, PICO_ERROR_INVALID_ARG};

use super::ds3231::Ds3231;
use super::lock;
use super::sh1106::Sh1106;

/// A device on an emulated bus, seen at the level of the I2C transfers.
pub(super) trait I2cDevice: Send {
    /// A write transfer addressed to the device, already ACKed
    fn write(&mut self, data: &[u8]);

    /// A read transfer addressed to the device, already ACKed
    fn read(&mut self, buffer: &mut [u8]);
}

type Bus = Vec<(u8, Box<dyn I2cDevice>)>;

const BUS_COUNT: usize = 2;

static BUSES: LazyLock<[Mutex<Bus>; BUS_COUNT]> = LazyLock::new(|| {
    [
        Mutex::new(alloc::vec![(Ds3231::I2C_ADDRESS, Box::new(Ds3231::new()) as Box<dyn I2cDevice>)]),
        Mutex::new(alloc::vec![(Sh1106::I2C_ADDRESS, Box::new(Sh1106::new()) as Box<dyn I2cDevice>)]),
    ]
});

/// Stands in for the `i2c_inst_t` of the pico-sdk: only its address is used,
/// as the handle the firmware passes back
static INSTANCES: [u8; BUS_COUNT] = [0, 1];

fn bus_of(i2c: *mut c_void) -> Option<&'static Mutex<Bus>> {
    INSTANCES
        .iter()
        .position(|instance| core::ptr::eq(instance, i2c as *const u8))
        .map(|index| &BUSES[index])
}

/// Runs `f` on the device at `addr`, returning `len` or the NACK error.
fn transfer(i2c: *mut c_void, addr: u8, len: usize, f: impl FnOnce(&mut dyn I2cDevice)) -> i32 {
    let Some(bus) = bus_of(i2c) else {
        return PICO_ERROR_INVALID_ARG as i32;
    };

    let mut bus = lock(bus);
    let Some((_, device)) = bus.iter_mut().find(|(address, _)| *address == addr) else {
        return PICO_ERROR_GENERIC as i32;
    };

    f(device.as_mut());
    len as i32
}

pub(in crate::drivers) unsafe fn hhg_i2c_instance(i2c_num: u8) -> *mut c_void {
    match INSTANCES.get(i2c_num as usize) {
        Some(instance) => instance as *const u8 as *mut c_void,
        None => core::ptr::null_mut(),
    }
}

pub(in crate::drivers) unsafe fn hhg_i2c_init(i2c: *mut c_void, baudrate: c_uint) -> c_uint {
    if bus_of(i2c).is_none() {
        return 1;
    }
    // Standard and fast mode rates are reached exactly from the 150 MHz clock
    baudrate
}

pub(in crate::drivers) unsafe fn hhg_i2c0_init_pins_with_func() {}

pub(in crate::drivers) unsafe fn hhg_i2c1_init_pins_with_func() {}

pub(in crate::drivers) unsafe fn hhg_i2c_write_blocking(i2c: *mut c_void, addr: u8, src: *const u8, len: usize, _nostop: bool) -> i32 {
    if src.is_null() && len > 0 {
        return PICO_ERROR_INVALID_ARG as i32;
    }
    let data = if len == 0 { &[][..] } else { unsafe { from_raw_parts(src, len) } };
    transfer(i2c, addr, len, |device| device.write(data))
}

pub(in crate::drivers) unsafe fn hhg_i2c_write_blocking_dma(i2c: *mut c_void, addr: u8, src: *const u8, len: usize, nostop: bool) -> i32 {
    if i2c.is_null() || src.is_null() || len == 0 {
        return PICO_ERROR_INVALID_ARG as i32;
    }
    unsafe { hhg_i2c_write_blocking(i2c, addr, src, len, nostop) }
}

pub(in crate::drivers) unsafe fn hhg_i2c_read_blocking(i2c: *mut c_void, addr: u8, dst: *mut u8, len: usize, _nostop: bool) -> i32 {
    if dst.is_null() && len > 0 {
        return PICO_ERROR_INVALID_ARG as i32;
    }
    let buffer = if len == 0 { &mut [][..] } else { unsafe { from_raw_parts_mut(dst, len) } };
    transfer(i2c, addr, len, |device| device.read(buffer))
}

pub(in crate::drivers) unsafe fn hhg_i2c_deinit(_i2c: *mut c_void) {}
