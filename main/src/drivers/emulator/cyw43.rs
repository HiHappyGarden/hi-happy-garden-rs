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

//! CYW43 radio and lwIP stack, with no access point in range.
//!
//! Twin of `src/pico/hhg-cyw43-wrapper.c` and `hhg-lwip.c`. The radio
//! initialises and reports `CYW43_LINK_NONET`, a join attempt times out and
//! lwIP never gets a link, the same as a board far from its WiFi: the
//! firmware runs its offline paths.

use core::ffi::{c_char, c_int, c_uchar, c_uint, c_ushort, c_void};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::drivers::pico::ffi::cyw43_status::{CYW43_LINK_DOWN, CYW43_LINK_NONET};
use crate::drivers::pico::ffi::err_enum::ERR_ARG;
use crate::drivers::pico::ffi::pico_error_codes::{PICO_ERROR_NO_DATA, PICO_ERROR_TIMEOUT};
use crate::drivers::pico::ffi::{ip_addr, pbuf, udp_pcb, udp_recv_fn};

/// How long a join attempt takes before giving up
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

static STA_MODE: AtomicBool = AtomicBool::new(false);

static JOIN_ATTEMPTED: AtomicBool = AtomicBool::new(false);

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_init_with_country(_country_code: c_uint) -> c_int {
    0
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_deinit() {
    STA_MODE.store(false, Ordering::Release);
    JOIN_ATTEMPTED.store(false, Ordering::Release);
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_enable_sta_mode() {
    STA_MODE.store(true, Ordering::Release);
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_disable_sta_mode() {
    STA_MODE.store(false, Ordering::Release);
    JOIN_ATTEMPTED.store(false, Ordering::Release);
}

pub(in crate::drivers) unsafe fn hhg_cyw43_wifi_link_status(_itf: c_int) -> c_int {
    if STA_MODE.load(Ordering::Acquire) && JOIN_ATTEMPTED.load(Ordering::Acquire) {
        CYW43_LINK_NONET
    } else {
        CYW43_LINK_DOWN
    }
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_wifi_connect(_ssid: *const c_char, _pw: *const c_char, _auth: c_uint) -> c_int {
    JOIN_ATTEMPTED.store(true, Ordering::Release);
    // The scan for the SSID takes time on the real radio: returning at once
    // would make the WiFi state machine spin
    std::thread::sleep(CONNECT_TIMEOUT);
    PICO_ERROR_TIMEOUT as c_int
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_poll() {}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_lwip_begin() {}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_lwip_end() {}

pub(in crate::drivers) unsafe fn hhg_cyw43_wifi_get_rssi(_rssi: *mut i32) -> i32 {
    // Not associated: the driver has no RSSI to report
    PICO_ERROR_NO_DATA as i32
}

pub(in crate::drivers) unsafe fn hhg_dhcp_get_ip_address() -> *const c_char {
    core::ptr::null()
}

pub(in crate::drivers) unsafe fn hhg_dhcp_get_binary_ip_address() -> c_uint {
    0
}

pub(in crate::drivers) unsafe fn hhg_dhcp_supplied_address() -> bool {
    false
}

pub(in crate::drivers) unsafe fn hhg_netif_is_link_up() -> c_uchar {
    0
}

pub(in crate::drivers) unsafe fn hhg_ip_addr_cmp(addr: *const ip_addr, addr2: *const ip_addr) -> i32 {
    match unsafe { (addr.as_ref(), addr2.as_ref()) } {
        (Some(a), Some(b)) => (a.addr == b.addr) as i32,
        _ => 0,
    }
}

pub(in crate::drivers) unsafe fn hhg_dns_gethostbyname(
    _hostname: *const c_char,
    _addr: *mut ip_addr,
    _dns_found_callback: extern "C" fn(name: *const c_char, ipaddr: *const ip_addr, callback_arg: *mut c_void),
    _callback_arg: *mut c_void,
) -> c_char {
    // No netif up: lwIP refuses the query without calling back
    ERR_ARG as i8 as c_char
}

pub(in crate::drivers) unsafe fn hhg_udp_new_ip_type(_type: c_uchar) -> *mut udp_pcb {
    null_mut()
}

pub(in crate::drivers) unsafe fn hhg_pbuf_alloc(_length: c_ushort) -> *mut pbuf {
    null_mut()
}

pub(in crate::drivers) unsafe fn hhg_pbuf_free(_p: *mut pbuf) -> c_uchar {
    0
}

pub(in crate::drivers) unsafe fn hhg_pbuf_copy_partial(_buf: *mut pbuf, _dataptr: *mut c_void, _len: u16, _offset: u16) -> u16 {
    0
}

pub(in crate::drivers) unsafe fn hhg_pbuf_get_at(_p: *const pbuf, _offset: u16) -> u8 {
    0
}

pub(in crate::drivers) unsafe fn hhg_udp_sendto(_pcb: *mut udp_pcb, _p: *mut pbuf, _ipaddr: *const ip_addr, _port: u16) -> i8 {
    ERR_ARG as i8
}

pub(in crate::drivers) unsafe fn hhg_udp_recv(_pcb: *mut udp_pcb, _recv: udp_recv_fn, _recv_arg: *mut c_void) {}
