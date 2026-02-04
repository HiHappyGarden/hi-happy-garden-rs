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

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;
use core::time::Duration;
use osal_rs::{log_error, log_info};
use osal_rs::os::{MutexFn, MutexGuard, System, Thread, ThreadFn};
use osal_rs::os::types::{StackType, TickType};
use osal_rs::utils::{ArcMux, Result};

use crate::traits::state::Initializable;
use crate::drivers::platform::ThreadPriority;
use crate::drivers::pico::wifi_cyw43::WIFI_FN;
use crate::traits::wifi::{OnWifiChangeStatus, SetOnWifiChangeStatus, WifiStatus};
use crate::traits::wifi::WifiStatus::{Connected, Connecting, Disabled, Disconnecting, Enabled, Enabling, Error};

const APP_TAG: &str = "WIFI";
const APP_THREAD_NAME: &str = "wifi_trd";
const APP_STACK_SIZE: StackType = 256;

static mut FSM_STATUS_CURRENT: WifiStatus = WifiStatus::Disabled;
static mut FSM_STATUS_OLD: WifiStatus = WifiStatus::Disabled;

pub enum Auth {
    Open,
    Web,
    Wpa,
    Wpa2,
    Wpa2Mixed,
    Wpa3,
    Wpa2Wpa3
}


pub struct WifiFn {
    pub init: fn() -> Result<(*mut c_void, i32)>,
    pub enable_sta_mode: fn(*mut c_void),
    pub disable_sta_mode: fn(*mut c_void),
    pub connect: fn(*mut c_void, auth: Auth, ssid: &[u8], password: &[u8]) -> Result<i32>,
    pub link_status: fn(*mut c_void) -> i32,
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

            unsafe {
                loop {
                    match FSM_STATUS_CURRENT {
                        Disabled => {
                            log_info!(APP_TAG, "WiFi FSM: Disabled -> Enabling");

                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Enabling;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },
                        Enabling => {
                            log_info!(APP_TAG, "WiFi FSM: Enabling -> Enabled");

                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Enabled;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },
                        Enabled => {
                            log_info!(APP_TAG, "WiFi FSM: Enabled -> Connecting");
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Connecting;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },
                        Connecting => {
                            log_info!(APP_TAG, "WiFi FSM: Connecting -> Connected");
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Connected;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },
                        Connected => {
                            log_info!(APP_TAG, "WiFi FSM: Connected -> Disconnecting");
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Disconnecting;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                        },
                        Disconnecting => {
                            log_info!(APP_TAG, "WiFi FSM: Disconnecting -> Disabled");
                            FSM_STATUS_OLD = FSM_STATUS_CURRENT;
                            FSM_STATUS_CURRENT = Disabled;
                            on_wifi_change_status.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
                            break;
                        },
                        Error => {
                            log_info!(APP_TAG, "WiFi FSM: Error -> Disabled");
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
    pub fn new() -> Self {
        use alloc::sync::Arc;
        use osal_rs::os::Mutex;
        
        Self {
            handle: null_mut(),
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, ThreadPriority::Normal),
        }
    }

    #[inline]
    pub fn get_link_status(&self) {
        (WIFI_FN.link_status)(self.handle);
    }

}
