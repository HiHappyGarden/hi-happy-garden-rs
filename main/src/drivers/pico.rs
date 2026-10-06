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

pub(super) mod ffi;
#[cfg(feature = "pico")]
mod fault;
pub(super) mod flash;
pub(super) mod gpio;
pub(super) mod hardware;
pub(super) mod hw_timer;
pub(super) mod i2c;
pub(super) mod lwip;
pub(super) mod mbedtls;
pub(super) mod rtc_ds3231;
pub(super) mod uart;
pub(super) mod wifi_cyw43;
