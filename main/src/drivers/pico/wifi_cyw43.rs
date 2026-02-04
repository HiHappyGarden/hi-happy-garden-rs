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
use alloc::ffi::CString;
use core::ffi::{c_uchar, c_void};
use core::ptr::null_mut;
use osal_rs::{log_info, to_c_str};
use osal_rs::utils::{Error, Result};
use crate::drivers::pico::ffi::{hhg_cyw43_arch_disable_sta_mode, hhg_cyw43_arch_enable_sta_mode, hhg_cyw43_arch_init, hhg_cyw43_wifi_link_status};
use crate::drivers::pico::ffi::cyw43_auth::{OPEN, WPA_TKIP_PSK, WPA2_AES_PSK, WPA2_MIXED_PSK, WPA3_SAE_AES_PSK, WPA3_WPA2_AES_PSK};
use crate::drivers::plt::ffi::{hhg_cyw43_arch_deinit, hhg_cyw43_arch_wifi_connect_async, CYW43Itf};
use crate::drivers::wifi::{Auth, WifiFn};

pub const WIFI_FN: WifiFn = WifiFn {
    init,
    enable_sta_mode,
    disable_sta_mode,
    connect,
    link_status,
    drop,
};

fn  init() -> Result<(*mut c_void, i32)> {

    let ret = unsafe { hhg_cyw43_arch_init() };
    if ret == 0 {
        Ok((null_mut(),0))
    } else {
        Err(Error::ReturnWithCode(ret))
    }


}

#[inline]
fn enable_sta_mode(_: *mut c_void) {
    unsafe { hhg_cyw43_arch_enable_sta_mode(); }
}

#[inline]
fn disable_sta_mode(_: *mut c_void) {
    unsafe { hhg_cyw43_arch_disable_sta_mode(); }
}

fn  connect(_: *mut c_void, auth: Auth, ssid: &[u8], password: &[u8]) -> Result<i32> {
    let pico_auth = match auth {
        Auth::Open => OPEN,
        Auth::Wpa => WPA_TKIP_PSK,
        Auth::Wpa2 => WPA2_AES_PSK,
        Auth::Wpa2Mixed => WPA2_MIXED_PSK,
        Auth::Wpa3 => WPA3_SAE_AES_PSK,
        Auth::Wpa2Wpa3 => WPA3_WPA2_AES_PSK,
        _ => return Err(Error::ReturnWithCode(-10))
    };

    let ret = unsafe { hhg_cyw43_arch_wifi_connect_async( CString::new(ssid).map_err(|_| Error::NullPtr )?.into_raw(),  CString::new(password).map_err(|_| Error::NullPtr )?.into_raw(), pico_auth) };
    if ret == 0 {
        Ok(0)
    } else {
        Err(Error::ReturnWithCode(ret))
    }
}

#[inline]
fn link_status(_: *mut c_void) -> i32 {
    unsafe { hhg_cyw43_wifi_link_status(CYW43Itf::STA as u32) }
}

#[inline]
fn drop(_: *mut c_void) {
    unsafe { hhg_cyw43_arch_deinit() }
}