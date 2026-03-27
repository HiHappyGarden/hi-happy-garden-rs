/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
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

use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use osal_rs::{log_debug, log_error, log_info, log_warning};
use osal_rs::os::{System, Thread, ThreadFn};
use osal_rs::os::types::StackType;
use osal_rs::utils::{AsSyncStr, Bytes, Result};
use osal_rs_serde::{Deserialize, Serialize};
use crate::drivers::gpio::Gpio;
use crate::drivers::pico::ffi::pico_error_codes::PICO_OK;
use crate::traits::state::Initializable;
use crate::drivers::platform::{GpioPeripheral, ThreadPriority};
use crate::drivers::pico::wifi_cyw43::WIFI_FN;
use crate::traits::wifi::{OnWifiChangeStatus, RSSIStatus::{self, *}, SetOnWifiChangeStatus, WifiStatus};
use crate::traits::wifi::WifiStatus::Disconnected;


const APP_TAG: &str = "WIFI";
const THREAD_NAME: &str = "wifi_trd";
const STACK_SIZE: StackType = 1_536; // 1.5KB stack size, adjust as needed
const MAX_ERROR: StackType = 5;

static mut SSID: Bytes<32> = Bytes::new();
static mut PASSWORD: Bytes<32> = Bytes::new();
static mut AUTH: Auth = Auth::Wpa2;
static ENABLED: AtomicBool = AtomicBool::new(false);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut FSM_STATUS_CURRENT: WifiStatus = WifiStatus::Disabled;
static mut FSM_STATUS_OLD: WifiStatus = WifiStatus::Disabled;

// Macro per semplificare il cambio di stato WiFi
macro_rules! transition_wifi_status {
    ($new_status:expr, $callback:expr) => {
        FSM_STATUS_OLD = FSM_STATUS_CURRENT;
        FSM_STATUS_CURRENT = $new_status;
        let _ = $callback.on_status_change(FSM_STATUS_OLD, FSM_STATUS_CURRENT);
    };
}

#[macro_export]
macro_rules! wifi_country {
    ($a:expr, $b:expr, $rev:expr) => {
        ($a as u32) | (($b as u32) << 8) | (($rev as u32) << 16)
    };
}

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LinkStatus {
    Down,
    WaitForIp,
    Up,
    BadAuth,
} 


pub struct WifiFn {
    pub init: fn(u32) -> Result<*mut c_void>,
    pub enable_sta_mode: fn(*mut c_void),
    pub disable_sta_mode: fn(*mut c_void),
    pub connect: fn(*mut c_void, ssid: &str, password: &str, auth: Auth) -> Result<i32>,
    pub link_status: fn(*mut c_void) -> LinkStatus,
    pub get_rssi: fn(*mut c_void) -> Result<i32>,
    pub drop: fn(*mut c_void),
}

pub struct Wifi {
    handle: *mut c_void,
    thread: Thread,
    thread_started: AtomicBool,
}

 unsafe impl Send for Wifi {}
 unsafe impl Sync for Wifi {}

impl Initializable for Wifi {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init wifi");

        (WIFI_FN.init)(wifi_country!('I', 'T', 0))?;
        (WIFI_FN.enable_sta_mode)(null_mut());
        INITIALIZED.store(true, Ordering::Release);

        Ok(())
    }
}

impl Drop for Wifi {
    fn drop(&mut self) {
        unsafe {FSM_STATUS_CURRENT = Disconnected };
        System::delay_with_to_tick(Duration::from_millis(200));
        (WIFI_FN.drop)(self.handle);
    }
}

impl SetOnWifiChangeStatus<'static> for Wifi {
    fn set_on_wifi_change_status(&mut self, on_wifi_change_status: &'static dyn OnWifiChangeStatus) {
        // Check if thread is already running
        if self.thread_started.load(Ordering::Acquire) {
            log_warning!(APP_TAG, "Wifi thread already started, ignoring new callback");
            return;
        }

        // Mark thread as started
        self.thread_started.store(true, Ordering::Release);

        let ret = self.thread.spawn_simple(move || {

            use WifiStatus::*;

            let mut count_error = 0;


            log_debug!(APP_TAG, "Start WIFI FSM");

            let mut link_status = LinkStatus::Down;

            let gpio = Gpio::new();

            let mut rssi_old : i8 = Unknown.into();

            unsafe {
                'no_rtc: loop {

                    
                        if !ENABLED.load(Ordering::Acquire) {
                            if FSM_STATUS_CURRENT != Disconnected && FSM_STATUS_CURRENT != Disabled {
                                transition_wifi_status!(Disconnected, on_wifi_change_status);
                            } else if FSM_STATUS_CURRENT == Disconnected {
                                transition_wifi_status!(Disconnected, on_wifi_change_status);
                                continue;
                            } if FSM_STATUS_CURRENT == Disabled {
                                on_wifi_change_status.on_rssi_change(RSSIStatus::Unknown);
                                System::delay_with_to_tick(Duration::from_millis(1_000));
                                continue;
                            }
                        }
                    

                    match FSM_STATUS_CURRENT {
                        Disabled => {

                            gpio.write(&GpioPeripheral::InternalLed, 0);

                            if !INITIALIZED.load(Ordering::Acquire) {
                                log_info!(APP_TAG, "WIFI init");
                                let _ = (WIFI_FN.init)(wifi_country!('I', 'T', 0));
                                (WIFI_FN.enable_sta_mode)(null_mut());
                                INITIALIZED.store(true, Ordering::Release);
                            }

                            System::delay_with_to_tick(Duration::from_millis(2_500));

                            count_error = 0;
                            transition_wifi_status!(Enabled, on_wifi_change_status);
                            on_wifi_change_status.on_rssi_change(RSSIStatus::Unknown);
                        },
                        Enabled => {
                            gpio.write(&GpioPeripheral::InternalLed, 0);

                            (WIFI_FN.enable_sta_mode)(null_mut());

                            count_error = 0;
                            transition_wifi_status!(Connecting, on_wifi_change_status);
                            on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                        },
                        Connecting => {
                            gpio.write(&GpioPeripheral::InternalLed, 0);

                            let ret = (WIFI_FN.connect)(null_mut(), (*&raw const SSID).as_str(), (*&raw const PASSWORD).as_str(), AUTH).unwrap_or(1);
                            if ret != PICO_OK as i32{
                                log_error!(APP_TAG, "Error connecting to WiFi, errno: {ret}");
                                transition_wifi_status!(Error, on_wifi_change_status);
                                continue 'no_rtc;
                            }

                            count_error = 0;
                            transition_wifi_status!(WaitForIp, on_wifi_change_status);
                            on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                            continue 'no_rtc;
                        },
                        WaitForIp => {  
                            gpio.write(&GpioPeripheral::InternalLed, 0);

                            link_status = (WIFI_FN.link_status)(null_mut());
                            match link_status {
                                LinkStatus::Up => {
                                    transition_wifi_status!(Connected, on_wifi_change_status);
                                    on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                                }
                                LinkStatus::WaitForIp => log_debug!(APP_TAG, "WiFi connected, waiting for DHCP..."),
                                LinkStatus::Down => {
                                    transition_wifi_status!(Error, on_wifi_change_status);
                                }
                                LinkStatus::BadAuth => {
                                    log_debug!(APP_TAG, "WiFi authentication failed");
                                    transition_wifi_status!(Error, on_wifi_change_status);
                                }
                            }
                    

                        }
                        Connected => {
                            link_status = (WIFI_FN.link_status)(null_mut());
                            match link_status {
                                LinkStatus::Up => {
                                    gpio.write(&GpioPeripheral::InternalLed, 1);

                                    let rssi =  if let Ok(rssi) = (WIFI_FN.get_rssi)(null_mut()) {
                                        match rssi {
                                            rssi if rssi >= -50 => Excellent,
                                            rssi if rssi >= -60 => Good,
                                            rssi if rssi >= -70 => Fair,
                                            rssi if rssi >= -80 => Weak,
                                            _ => NoSignal,
                                        }
                                    } else {
                                        Unknown.into()
                                    };

                                    if rssi_old != rssi.into() {
                                        on_wifi_change_status.on_rssi_change(rssi);
                                        rssi_old = rssi.into();
                                    }

                                    System::delay_with_to_tick(Duration::from_millis(500));
                                }
                                LinkStatus::WaitForIp => {
                                    transition_wifi_status!(WaitForIp, on_wifi_change_status);
                                }
                                LinkStatus::Down => {
                                    transition_wifi_status!(Error, on_wifi_change_status);
                                }
                                LinkStatus::BadAuth => {
                                    log_debug!(APP_TAG, "WiFi authentication failed");
                                    transition_wifi_status!(Error, on_wifi_change_status);
                                }
                            }
                            
                        },
                        Disconnected => {
                            (WIFI_FN.disable_sta_mode)(null_mut());
                            let _ = (WIFI_FN.drop)(null_mut());

                            INITIALIZED.store(false, Ordering::Release);

                            System::delay_with_to_tick(Duration::from_secs(25));

                            on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);

                            gpio.write(&GpioPeripheral::InternalLed, 0);

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
                                FSM_STATUS_CURRENT = if link_status == LinkStatus::BadAuth { Disconnected } else { Resetting };
                                let _ = on_wifi_change_status.on_status_change(Error, FSM_STATUS_CURRENT);
                            }
                        },
                        Resetting => {
                            log_warning!(APP_TAG,"Resetting WiFi wait 5 seconds...");

                            gpio.write(&GpioPeripheral::InternalLed, 0);

                            (WIFI_FN.disable_sta_mode)(null_mut());
                            let _ = (WIFI_FN.drop)(null_mut());

                            INITIALIZED.store(false, Ordering::Release);

                            System::delay_with_to_tick(Duration::from_millis(2_500));

                            transition_wifi_status!(Disabled, on_wifi_change_status);
                        },


                    }


                    System::delay_with_to_tick(Duration::from_millis(100));
                }
            }
        });

        if let Err(e) = ret {
            log_error!(APP_TAG, "Error spawning wifi thread: {:?}", e);
            self.thread_started.store(false, Ordering::Release);
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
            ENABLED.store(enabled, Ordering::Release);
        }
    }

    pub fn new() -> Self {
        Self {
            handle: null_mut(),
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal),
            thread_started: AtomicBool::new(false),
        }
    }

}
