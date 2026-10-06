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

//! Pico 2 W board emulated on a POSIX host.
//!
//! On the board the platform layer of [`crate::drivers::pico`] calls the
//! `hhg_*` C wrappers of `src/pico`, which drive the pico-sdk. Here the same
//! functions, with the same signatures, are implemented by host models, so
//! everything above them (pico glue, drivers, apps) runs unchanged on
//! osal-rs `posix`:
//!
//! | Board                      | Emulator                                              |
//! |----------------------------|-------------------------------------------------------|
//! | GPIO / PWM / ADC           | [`gpio`]: in-memory pin bank, inputs driven by `gpio::drive_input` |
//! | UART0                      | [`uart`]: TX on stdout, RX from stdin                 |
//! | I2C0 + DS3231, I2C1 + SH1106 | [`i2c`], [`ds3231`], [`sh1106`]: register level device models |
//! | littlefs on flash          | littlefs on a RAM image, `src/emulator/hhg-lfs-wrapper.c` |
//! | mbedtls AES, SHA-256 accel | [`crypto`]: RustCrypto `aes` and `sha2`               |
//! | POWMAN, unique id, reset, timers | [`system`]                                      |
//! | CYW43 + lwIP               | [`cyw43`]: radio present, no network in range          |
//!
//! The pin map, the I2C addresses and the flash geometry are the board ones:
//! the emulator is the same board, not a different platform.

// The whole `hhg_*` API is mirrored, including the functions the firmware
// does not call (yet), so the two layers stay interchangeable
#![allow(dead_code)]

mod crypto;
mod cyw43;
mod ds3231;
mod gpio;
mod i2c;
mod sh1106;
mod system;
mod uart;

use alloc::ffi::CString;
use core::ffi::{c_char, c_int};
use std::sync::{Mutex, MutexGuard};

use osal_rs::utils::{Error, Result};

/// The `hhg_*` functions, re-exported by `pico::ffi` in place of the C ones.
pub(in crate::drivers) mod hal {
    pub(in crate::drivers) use super::crypto::*;
    pub(in crate::drivers) use super::cyw43::*;
    pub(in crate::drivers) use super::gpio::*;
    pub(in crate::drivers) use super::i2c::*;
    pub(in crate::drivers) use super::system::*;
    pub(in crate::drivers) use super::uart::*;
}

pub(crate) use system::{get_g_setup_called, print_systick_status};

unsafe extern "C" {
    fn hhg_emulator_flash_set_image(path: *const c_char) -> c_int;
}

/// Mirrors the emulated flash partition to the file at `path`.
///
/// Must be called before the firmware mounts the filesystem. A missing or
/// short file is filled with erased flash, which the firmware formats on
/// mount like a blank chip; an existing image is mounted as it is, so
/// configuration and data survive an emulator restart.
///
/// Without a call the flash lives in RAM only and starts blank every run.
pub fn set_flash_image(path: &str) -> Result<()> {
    let path = CString::new(path).map_err(|_| Error::InvalidType)?;
    let ret = unsafe { hhg_emulator_flash_set_image(path.as_ptr()) };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}

/// Locks a model state, ignoring poisoning.
///
/// A panic in a firmware thread must not take the whole emulated board down
/// with it: on the real board the peripherals keep working too.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
