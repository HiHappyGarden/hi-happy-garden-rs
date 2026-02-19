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
#![allow(dead_code)]

use alloc::sync::Arc;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::time::Duration;
use osal_rs::{log_debug, log_error, log_info, log_warning};
use osal_rs::os::{AsSyncStr, Mutex, MutexFn, System, Thread, ThreadFn};
use osal_rs::os::types::StackType;
use osal_rs::utils::{Bytes, Result};
use osal_rs_serde::{Deserialize, Serialize};
use crate::cyw43_country;
use crate::drivers::network::Network;
use crate::drivers::pico::ffi::CYW43Itf;
use crate::drivers::pico::ffi::pico_error_codes::PICO_OK;
use crate::traits::state::Initializable;
use crate::drivers::platform::ThreadPriority;
use crate::drivers::pico::wifi_cyw43::WIFI_FN;
use crate::traits::wifi::{OnWifiChangeStatus, SetOnWifiChangeStatus, WifiStatus};
use crate::traits::wifi::WifiStatus::Disconnecting;

mod cyw43_status {
    ///< link is down
    pub const CYW43_LINK_DOWN   : i32 = 0;
    ///< Connected to wifi
    pub const CYW43_LINK_JOIN   : i32 = 1;
    ///< Connected to wifi, but no IP address
    pub const CYW43_LINK_NOIP   : i32 = 2;
    ///< Connected to wifi with an IP address
    pub const CYW43_LINK_UP     : i32 = 3;
    ///< Connection failed
    pub const CYW43_LINK_FAIL   : i32 = -1;
    ///< No matching SSID found (could be out of range, or down)
    pub const CYW43_LINK_NONET  : i32 = -2;
    ///< Authenticatation failure
    pub const CYW43_LINK_BADAUTH: i32 = -3;
}


const APP_TAG: &str = "WIFI";
const APP_THREAD_NAME: &str = "wifi_trd";
const APP_STACK_SIZE: StackType = 1024;
const MAX_ERROR: StackType = 5;

static mut SSID: Bytes<32> = Bytes::new();
static mut PASSWORD: Bytes<32> = Bytes::new();
static mut AUTH: Auth = Auth::Wpa2;
static mut ENABLED: bool = false;

static mut FSM_STATUS_CURRENT: WifiStatus = WifiStatus::Disabled;
static mut FSM_STATUS_OLD: WifiStatus = WifiStatus::Disabled;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Auth {
    Open = 0,
    Web = 1,
    Wpa = 2,
    Wpa2 = 3,
    Wpa2Mixed = 4,
    Wpa3 = 5,
    Wpa2Wpa3 = 6
}

impl Into<u8> for Auth {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for Auth {
    fn from(value: u8) -> Self {
        match value {
            0 => Auth::Open,
            1 => Auth::Web,
            2 => Auth::Wpa,
            3 => Auth::Wpa2,
            4 => Auth::Wpa2Mixed,
            5 => Auth::Wpa3,
            6 => Auth::Wpa2Wpa3,
            _ => Auth::Open, // Default to Open if unknown value
        }
    }
}

impl Serialize for Auth {
    #[inline]
    fn serialize<S: osal_rs_serde::Serializer>(&self, name: &str, serializer: &mut S) -> core::result::Result<(), S::Error> {
        serializer.serialize_u8(name, *self as u8)?;
        Ok(())
    }
}

impl Deserialize for Auth {
    #[inline]
    fn deserialize<D: osal_rs_serde::Deserializer>(deserializer: &mut D, name: &str) -> core::result::Result<Self, D::Error> {
        Ok(Auth::from(deserializer.deserialize_u8(name)?))
    }
}

pub struct WifiFn {
    pub init: fn(u32) -> Result<*mut c_void>,
    pub enable_sta_mode: fn(*mut c_void),
    pub disable_sta_mode: fn(*mut c_void),
    pub connect: fn(*mut c_void, ssid: &str, password: &str, auth: Auth) -> Result<i32>,
    pub link_status: fn(*mut c_void, conf: i32) -> i32,
    pub drop: fn(*mut c_void),
}

pub struct Wifi {
    handle: *mut c_void,
    thread: Thread,
    initialized: Arc<Mutex<bool>>,
}

 unsafe impl Send for Wifi {}
 unsafe impl Sync for Wifi {}

impl Initializable for Wifi {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init wifi");

        (WIFI_FN.init)(cyw43_country!('I', 'T', 0))?;
        (WIFI_FN.enable_sta_mode)(null_mut());
        *self.initialized.lock()? = true;

        Ok(())
    }
}

impl Drop for Wifi {
    fn drop(&mut self) {
        unsafe {FSM_STATUS_CURRENT = Disconnecting };
        System::delay_with_to_tick(Duration::from_millis(200));
        (WIFI_FN.drop)(self.handle);
    }
}

impl SetOnWifiChangeStatus<'static> for Wifi {
    fn set_on_wifi_change_status(&mut self, on_wifi_change_status: &'static dyn OnWifiChangeStatus) {


        let initialized = Arc::clone(&self.initialized);

        let ret = self.thread.spawn_simple(move || {

            use WifiStatus::*;

            let mut count_error = 0;


            log_debug!(APP_TAG, "Start WIFI FSM");

            unsafe {
                'no_rtc: loop {

                    match FSM_STATUS_CURRENT {
                        Disabled => {

                            if !*initialized.lock().unwrap() {
                                log_info!(APP_TAG, "WIFI init");
                                let _ = (WIFI_FN.init)(cyw43_country!('I', 'T', 0));
                                (WIFI_FN.enable_sta_mode)(null_mut());
                                *initialized.lock().unwrap() = true;
                            }

                            System::delay_with_to_tick(Duration::from_millis(2_500));

                            count_error = 0;
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Enabled;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },
                        Enabled => {

                            (WIFI_FN.enable_sta_mode)(null_mut());

                            count_error = 0;
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Connecting;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },
                        Connecting => {

                            let ret = (WIFI_FN.connect)(null_mut(), (*&raw const SSID).as_str(), (*&raw const PASSWORD).as_str(), AUTH).unwrap_or(1);
                            if ret != PICO_OK as i32{
                                log_error!(APP_TAG, "Error connecting to WiFi, errno: {ret}");
                                FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                                FSM_STATUS_CURRENT = Error;
                                continue 'no_rtc;
                            }

                            count_error = 0;
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Connected;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                            continue 'no_rtc;
                        },
                        Connected => {
                            use cyw43_status::*;

                            // Update FSM_STATUS_OLD on first entry to Connected to avoid repeated callbacks
                            if FSM_STATUS_OLD != Connected {
                                FSM_STATUS_OLD = Connected;
                            }

                            // Poll the network stack to process DHCP packets (helps in FreeRTOS mode)
                           // hhg_cyw43_arch_poll();

                            // Check if DHCP has assigned an IP regardless of link status
                            // Some APs (like iPhone hotspot) may report NOIP even when DHCP is working
                            if Network::dhcp_supplied_address() {
                                // DHCP complete! Log IP only once
                                static mut IP_LOGGED: bool = false;
                                if !IP_LOGGED {
                                    let ip = Network::dhcp_get_ip_address();
                                    let ip_str = ip.as_str();
                                    log_info!(APP_TAG, "Connected to WiFi with IP: {}", ip_str);
                                    IP_LOGGED = true;
                                }
                            } else {
                                // No IP yet, check link status for more details
                                let link_status = (WIFI_FN.link_status)(null_mut(), CYW43Itf::STA as i32);

                                match link_status {
                                    CYW43_LINK_UP | CYW43_LINK_JOIN | CYW43_LINK_NOIP => {
                                        // Link is good, just waiting for DHCP
                                        log_debug!(APP_TAG, "WiFi connected (status: {}), waiting for DHCP...", link_status);
                                    },
                                    _ => {
                                        log_error!(APP_TAG, "WiFi link is down or failed (status: {})", link_status);
                                        FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                                        FSM_STATUS_CURRENT = Error;
                                        on_wifi_change_status.on_status_change(FSM_STATUS_CURRENT, Error);
                                        continue 'no_rtc;
                                    }
                                }
                            }
                        },
                        Disconnecting => {
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Disabled;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                            break;
                        },
                        Error => {
                            if count_error < MAX_ERROR {
                                count_error += 1;
                                log_error!(APP_TAG, "Error {}/{} retry...", count_error, MAX_ERROR);
                                FSM_STATUS_CURRENT = FSM_STATUS_OLD;
                                FSM_STATUS_OLD = Error;
                                System::delay_with_to_tick(Duration::from_millis(1_000));
                            } else {
                                log_error!(APP_TAG, "Resetting WiFi after {} errors", MAX_ERROR);
                                FSM_STATUS_OLD = Error;
                                FSM_STATUS_CURRENT = Resetting;
                                on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                            }
                        },
                        Resetting => {
                            log_warning!(APP_TAG,"Resetting WiFi wait 5 seconds...");


                            (WIFI_FN.disable_sta_mode)(null_mut());
                            let _ = (WIFI_FN.drop)(null_mut());

                            *initialized.lock().unwrap() = false;

                            System::delay_with_to_tick(Duration::from_millis(2_500));

                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Disabled;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },


                    }

                    System::delay_with_to_tick(Duration::from_millis(100));
                }
            }
        });

        if let Err(e) = ret {
            log_error!(APP_TAG, "Error spawning wifi thread: {:?}", e);
        }
    }
}

impl Wifi {

    pub fn set_config(
        ssid: Bytes<32>,
        password: Bytes<32>,
        auth: Auth,
        enabled: bool,
    ) {
        unsafe {
            SSID = ssid;
            PASSWORD = password;
            AUTH = auth;
            ENABLED = enabled;
        }
    }

    pub fn new() -> Self {
        Self {
            handle: null_mut(),
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, ThreadPriority::Normal),
            initialized: Mutex::new_arc(false),
        }
    }

    #[inline]
    pub fn get_link_status(&self) {
        (WIFI_FN.link_status)(self.handle, CYW43Itf::STA as i32);
    }


}
