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

//! CYW43 WiFi radio.
//!
//! Twin of `src/pico/hhg-cyw43-wrapper.c`. Online (the default) the board
//! sits next to an access point that accepts any credentials, and its
//! traffic goes out through the host network ([`super::lwip`]). Offline
//! (`--offline`), or after `wifi down` on the control channel, no access
//! point is in range: a join times out and the link reports
//! `CYW43_LINK_NONET`, the firmware runs its offline paths.

use core::ffi::{c_char, c_int, c_uint};
use core::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::drivers::pico::ffi::cyw43_status::{CYW43_LINK_DOWN, CYW43_LINK_JOIN, CYW43_LINK_NONET, CYW43_LINK_UP};
use crate::drivers::pico::ffi::pico_error_codes::{PICO_ERROR_NO_DATA, PICO_ERROR_TIMEOUT};

/// How long associating with the access point takes
const JOIN_TIME: Duration = Duration::from_millis(300);

/// How long a scan for an absent access point lasts before giving up
const JOIN_TIMEOUT: Duration = Duration::from_millis(500);

/// Signal of the emulated access point, a good one
const RSSI_DBM: i32 = -55;

/// The host network is used (`--offline` clears it)
static ONLINE: AtomicBool = AtomicBool::new(true);

/// The access point is in range (`wifi up` / `wifi down` on the control channel)
static AP_IN_RANGE: AtomicBool = AtomicBool::new(true);

static STA_MODE: AtomicBool = AtomicBool::new(false);

static JOIN_ATTEMPTED: AtomicBool = AtomicBool::new(false);

static JOINED: AtomicBool = AtomicBool::new(false);

pub(super) fn set_online(online: bool) {
    ONLINE.store(online, Ordering::Release);
    if !online {
        JOINED.store(false, Ordering::Release);
    }
}

/// Brings the access point in or out of range; out of range drops the
/// association, as walking away from it would.
pub(super) fn set_ap_in_range(in_range: bool) {
    AP_IN_RANGE.store(in_range, Ordering::Release);
    if !in_range {
        JOINED.store(false, Ordering::Release);
    }
}

fn ap_reachable() -> bool {
    ONLINE.load(Ordering::Acquire) && AP_IN_RANGE.load(Ordering::Acquire)
}

/// Associated with the access point, with an address: what lwIP needs to
/// put traffic on the host network.
pub(super) fn is_joined() -> bool {
    JOINED.load(Ordering::Acquire) && STA_MODE.load(Ordering::Acquire)
}

/// One line for the control channel `status`.
pub(super) fn describe() -> &'static str {
    match (ONLINE.load(Ordering::Acquire), AP_IN_RANGE.load(Ordering::Acquire), is_joined()) {
        (false, _, _) => "offline",
        (true, false, _) => "access point out of range",
        (true, true, true) => "connected",
        (true, true, false) => "access point in range, not connected",
    }
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_init_with_country(_country_code: c_uint) -> c_int {
    0
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_deinit() {
    STA_MODE.store(false, Ordering::Release);
    JOIN_ATTEMPTED.store(false, Ordering::Release);
    JOINED.store(false, Ordering::Release);
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_enable_sta_mode() {
    STA_MODE.store(true, Ordering::Release);
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_disable_sta_mode() {
    STA_MODE.store(false, Ordering::Release);
    JOIN_ATTEMPTED.store(false, Ordering::Release);
    JOINED.store(false, Ordering::Release);
}

pub(in crate::drivers) unsafe fn hhg_cyw43_wifi_link_status(_itf: c_int) -> c_int {
    if !STA_MODE.load(Ordering::Acquire) {
        return CYW43_LINK_DOWN;
    }
    if is_joined() {
        return CYW43_LINK_UP;
    }
    if !JOIN_ATTEMPTED.load(Ordering::Acquire) {
        return CYW43_LINK_DOWN;
    }
    if ap_reachable() { CYW43_LINK_JOIN } else { CYW43_LINK_NONET }
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_wifi_connect(_ssid: *const c_char, _pw: *const c_char, _auth: c_uint) -> c_int {
    JOIN_ATTEMPTED.store(true, Ordering::Release);

    // Joining takes time on the real radio too: returning at once would make
    // the WiFi state machine spin
    if !ap_reachable() {
        std::thread::sleep(JOIN_TIMEOUT);
        return PICO_ERROR_TIMEOUT as c_int;
    }

    std::thread::sleep(JOIN_TIME);
    JOINED.store(ap_reachable(), Ordering::Release);
    if is_joined() { 0 } else { PICO_ERROR_TIMEOUT as c_int }
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_poll() {}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_lwip_begin() {}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_lwip_end() {}

pub(in crate::drivers) unsafe fn hhg_cyw43_wifi_get_rssi(rssi: *mut i32) -> i32 {
    if !is_joined() || rssi.is_null() {
        // Not associated: the driver has no RSSI to report
        return PICO_ERROR_NO_DATA as i32;
    }
    unsafe { *rssi = RSSI_DBM };
    0
}
