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

use core::ffi::c_void;
use core::ptr::null_mut;
use core::time::Duration;
use osal_rs::{log_debug, log_error, log_info};
use osal_rs::os::{AsSyncStr, System, Thread, ThreadFn};
use osal_rs::os::types::StackType;
use osal_rs::utils::{Result};
use osal_rs_serde::{Deserialize, Serialize};
use crate::apps::config::Config;
use crate::drivers::date_time::DateTime;
use crate::drivers::pico::ffi::CYW43Itf;
use crate::traits::state::Initializable;
use crate::drivers::platform::ThreadPriority;
use crate::drivers::pico::wifi_cyw43::WIFI_FN;
use crate::drivers::rtc::RTC;
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
const APP_STACK_SIZE: StackType = 512;

const MAX_ERROR: StackType = 5;

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
    pub init: fn() -> Result<*mut c_void>,
    pub enable_sta_mode: fn(*mut c_void),
    pub disable_sta_mode: fn(*mut c_void),
    pub connect: fn(*mut c_void, ssid: &str, password: &str, auth: Auth) -> Result<i32>,
    pub link_status: fn(*mut c_void, conf: i32) -> i32,
    pub drop: fn(*mut c_void),
}

pub struct Wifi {
    handle: *mut c_void,
    thread: Thread,
}

 unsafe impl Send for Wifi {}
 unsafe impl Sync for Wifi {}

impl Initializable for Wifi {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init wifi");

        (WIFI_FN.init)()?;

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

        let ret = self.thread.spawn_simple(move || {

            use WifiStatus::*;

            let mut count_error = 0;
            let mut attempts = 0;

            let rtc = RTC::new();

            unsafe {
                'no_rtc: loop {


                    // let now = rtc.get_timestamp().unwrap_or_else(|e| {
                    //     log_error!(APP_TAG, "Error getting timestamp: {:?}", e);
                    //     0
                    // });
                    //
                    // if now == 0 {
                    //     System::delay_with_to_tick(Duration::from_millis(1_000));
                    //     continue 'no_rtc;
                    // }

                    match FSM_STATUS_CURRENT {
                        Disabled => {

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
                            

                            let config = Config::new().get_wifi_config();

                                
                            if !Self::handle_connection_result ((WIFI_FN.connect)(null_mut(), config.get_ssid().as_str(), config.get_password().as_str(), config.get_auth())) {
                                log_error!(APP_TAG, "Error connecting to WiFi, count_error: {}", count_error);
                                FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                                FSM_STATUS_CURRENT = Error;
                                continue 'no_rtc;
                            }

                            count_error = 0;
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Connected;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },
                        Connected => {



                            if (WIFI_FN.link_status)(null_mut(), CYW43Itf::STA as i32) != cyw43_status::CYW43_LINK_UP {
                                FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                                FSM_STATUS_CURRENT = Error;
                                continue 'no_rtc;
                            }


                            //FSM_STATUS_CURRENT = Disconnecting;
                            if FSM_STATUS_OLD != FSM_STATUS_CURRENT {
                                on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                            }
                        },
                        Disconnecting => {
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Disabled;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                            break;
                        },
                        Error => {
                            log_debug!(APP_TAG, "Error!!!");
                            if count_error < MAX_ERROR {
                                count_error += 1;
                                log_debug!(APP_TAG, "Error {}/{} retry...", count_error, MAX_ERROR);
                                FSM_STATUS_CURRENT = FSM_STATUS_OLD;
                                FSM_STATUS_OLD = Resetting;
                                System::delay_with_to_tick(Duration::from_millis(500));
                            } else {
                                FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                                FSM_STATUS_CURRENT = Connecting;
                                on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                            }
                        },
                        Resetting => {
                            panic!("Resetting WiFi..."); // TODO: implement reset logic
                            // log_debug!(APP_TAG, "Resetting...");
                            // FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            // FSM_STATUS_CURRENT = Disabled;
                            // on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },


                    }

                    if FSM_STATUS_CURRENT != FSM_STATUS_OLD {
                        log_debug!(APP_TAG, "FSM_STATUS_OLD: {} -> FSM_STATUS_CURRENT: {}", *&raw const FSM_STATUS_OLD, *&raw const FSM_STATUS_CURRENT);
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
    pub fn new() -> Self {

        Self {
            handle: null_mut(),
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, ThreadPriority::Normal),
        }
    }

    #[inline]
    pub fn get_link_status(&self) {
        (WIFI_FN.link_status)(self.handle, CYW43Itf::STA as i32);
    }


    fn handle_connection_result(result: Result<i32>) -> bool {
        use cyw43_status::*;
        use osal_rs::utils::Error::ReturnWithCode;

        match &result {
            Err(ReturnWithCode(_code @ CYW43_LINK_FAIL))  => log_error!(APP_TAG, "Connection failed"),
            Err(ReturnWithCode(_code @ CYW43_LINK_NONET)) => log_error!(APP_TAG, "No matching SSID found (could be out of range, or down)"),
            Err(ReturnWithCode(_code @ CYW43_LINK_BADAUTH)) => log_error!(APP_TAG, "Authenticatation failure"),
            Ok(_e @ CYW43_LINK_JOIN) => log_info!(APP_TAG, "Connected to WiFi"),
            Ok(_e @ CYW43_LINK_NOIP) => log_info!(APP_TAG, "Connected to WiFi but no IP address"),
            Ok(_e @ CYW43_LINK_UP) => log_info!(APP_TAG, "Connected to WiFi with IP address"),
            Err(e) => log_error!(APP_TAG, "Error connecting to WiFi: {:?}", e),
            _ => log_error!(APP_TAG, "Unknown error connecting to WiFi"),
        }

        if result.is_err() {
            false
        } else {
            true
        }
    }

}
