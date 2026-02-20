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
use alloc::format;
use core::ffi::c_void;
use core::ptr::null_mut;
use osal_rs::utils::{Error, Result};
use crate::drivers::pico::ffi::{hhg_cyw43_arch_disable_sta_mode, hhg_cyw43_arch_enable_sta_mode, hhg_cyw43_arch_wifi_connect, hhg_cyw43_wifi_link_status, hhg_cyw43_arch_deinit, hhg_cyw43_arch_init_with_country};
use crate::drivers::pico::ffi::cyw43_auth::{OPEN, WPA_TKIP_PSK, WPA2_AES_PSK, WPA2_MIXED_PSK, WPA3_SAE_AES_PSK, WPA3_WPA2_AES_PSK};
use crate::drivers::wifi::{Auth, LinkStatus, WifiFn};

pub const WIFI_FN: WifiFn = WifiFn {
    init,
    enable_sta_mode,
    disable_sta_mode,
    connect,
    link_status,
    drop,
};

fn  init(country_code: u32) -> Result<*mut c_void> {

    let ret = unsafe { hhg_cyw43_arch_init_with_country(country_code) };
    if ret == 0 {
        Ok(null_mut())
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

fn  connect(_: *mut c_void, ssid: &str, password: &str, auth: Auth) -> Result<i32> {
    let pico_auth = match auth {
        Auth::Open => OPEN,
        Auth::Wpa => WPA_TKIP_PSK,
        Auth::Wpa2 => WPA2_AES_PSK,
        Auth::Wpa2Mixed => WPA2_MIXED_PSK,
        Auth::Wpa3 => WPA3_SAE_AES_PSK,
        Auth::Wpa2Wpa3 => WPA3_WPA2_AES_PSK,
        _ => return Err(Error::ReturnWithCode(-10))
    };

    let ssid = CString::new(ssid).map_err(|e| Error::UnhandledOwned(format!("SSID contains null byte: {}", e)))?;
    let password = CString::new(password).map_err(|e| Error::UnhandledOwned(format!("Password contains null byte: {}", e)))?;

    let ret = unsafe { hhg_cyw43_arch_wifi_connect(ssid.as_ptr(), password.as_ptr(), pico_auth) };
    if ret == 0 {
        Ok(0)
    } else {
        Err(Error::ReturnWithCode(ret))
    }
}

#[inline]
fn link_status(_: *mut c_void) -> LinkStatus {
    use crate::drivers::pico::ffi::cyw43_status::*;
    use LinkStatus::*;
    
    match unsafe { hhg_cyw43_wifi_link_status(0) } {
        CYW43_LINK_UP => Up,
        CYW43_LINK_DOWN | CYW43_LINK_JOIN | CYW43_LINK_NOIP => WaitForIp,    
        CYW43_LINK_FAIL | CYW43_LINK_NONET => Down,
        CYW43_LINK_BADAUTH => BadAuth,
        _ => Down, // Default to Down for unknown statuses
    }
}

#[inline]
fn drop(_: *mut c_void) {
    unsafe { hhg_cyw43_arch_deinit() }
}