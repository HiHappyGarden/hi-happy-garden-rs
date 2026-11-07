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

use alloc::sync::Arc;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use core::time::Duration;
use osal_rs::{log_debug, log_error, log_info, log_warning};
use osal_rs::os::{RawMutex, RawMutexGuard, System, Thread, ThreadFn};
use osal_rs::os::types::StackType;
use osal_rs::utils::{Bytes, OnceBox, Result};
use osal_rs_serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::drivers::gpio::Gpio;
use crate::drivers::pico::ffi::pico_error_codes::PICO_OK;
use crate::traits::state::Initializable;
use crate::drivers::platform::{GpioPeripheral, ThreadPriority};
use crate::drivers::pico::wifi_cyw43::WIFI_FN;
use crate::traits::wifi::{OnWifiChangeStatus, RSSIStatus::{self, *}, SetOnWifiChangeStatus, WifiStatus};
use crate::traits::wifi::WifiStatus::Disconnected;


const APP_TAG: &str = "WIFI";
const THREAD_NAME: &str = "wifi_thr";
const STACK_SIZE: StackType = 2_560; 
const MAX_ERROR: StackType = 5;

enum WifiFsmControl {
    Next,
    Continue,
    Break,
}

// Written by `set_config` (parser, display), read by the WiFi thread: only under `MUTEX`
static mut SSID: Bytes<32> = Bytes::new();
static mut PASSWORD: Bytes<32> = Bytes::new();
static mut AUTH: Auth = Auth::Wpa2;
static MUTEX: OnceBox<RawMutex> = OnceBox::new();
static ENABLED: AtomicBool = AtomicBool::new(false);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// `WifiStatus` as `u8`: the WiFi thread runs the FSM, `Drop` can stop it
static FSM_STATUS_CURRENT: AtomicU8 = AtomicU8::new(WifiStatus::Disabled as u8);
static FSM_STATUS_OLD: AtomicU8 = AtomicU8::new(WifiStatus::Disabled as u8);

#[inline]
fn credentials_lock() -> RawMutexGuard<RawMutex> {
    RawMutexGuard::acquire(MUTEX.get_or_try_init(RawMutex::new).expect("Failed to create WiFi mutex"))
}

#[inline]
fn status_current() -> WifiStatus {
    WifiStatus::from(FSM_STATUS_CURRENT.load(Ordering::Acquire))
}

#[inline]
fn status_old() -> WifiStatus {
    WifiStatus::from(FSM_STATUS_OLD.load(Ordering::Acquire))
}

// Macro per semplificare il cambio di stato WiFi
macro_rules! transition_wifi_status {
    ($new_status:expr, $callback:expr) => {
        let new_status: WifiStatus = $new_status;
        let old_status = WifiStatus::from(FSM_STATUS_CURRENT.swap(new_status as u8, Ordering::AcqRel));
        FSM_STATUS_OLD.store(old_status as u8, Ordering::Release);
        let _ = $callback.on_status_change(old_status, new_status);
    };
}

#[macro_export]
macro_rules! wifi_country {
    ($a:expr, $b:expr, $rev:expr) => {
        ($a as u32) | (($b as u32) << 8) | (($rev as u32) << 16)
    };
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    fn serialize<S: Serializer>(&self, name: &str, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_u8(name, *self as u8)?;
        Ok(())
    }
}

impl Deserialize for Auth {
    #[inline]
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> Result<Self, D::Error> {
        Ok(Auth::from(deserializer.deserialize_u8(name)?))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LinkStatus {
    Down,
    LinkNoNet, // Link is up but no network connectivity may exist, especially on non2.4GHz networks
    WaitForIp,
    Up,
    BadAuth,
} 


pub(in crate::drivers) struct WifiFn {
    pub(in crate::drivers) init: fn(u32) -> Result<*mut c_void>,
    pub(in crate::drivers) enable_sta_mode: fn(*mut c_void),
    pub(in crate::drivers) disable_sta_mode: fn(*mut c_void),
    pub(in crate::drivers) connect: fn(*mut c_void, ssid: &str, password: &str, auth: Auth) -> Result<i32>,
    pub(in crate::drivers) link_status: fn(*mut c_void) -> LinkStatus,
    pub(in crate::drivers) get_rssi: fn(*mut c_void) -> Result<i32>,
    pub(in crate::drivers) drop: fn(*mut c_void),
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
        FSM_STATUS_CURRENT.store(Disconnected as u8, Ordering::Release);
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

            let gpio = Gpio::shared();

            let mut rssi_old : i8 = Unknown.into();

            let mut led_on = true;

            {
                'no_rtc: loop {
                    if !ENABLED.load(Ordering::Acquire) {
                        if status_current() != Disconnected && status_current() != Disabled {
                            transition_wifi_status!(Disconnected, on_wifi_change_status);
                        } else if status_current() == Disconnected {
                            transition_wifi_status!(Disconnected, on_wifi_change_status);
                            continue;
                        } if status_current() == Disabled {
                            on_wifi_change_status.on_rssi_change(RSSIStatus::Unknown);
                            System::delay_with_to_tick(Duration::from_millis(1_000));
                            continue;
                        }
                    }
                    

                    let ctrl = match status_current() {
                        Disabled        => Self::handle_disabled(&gpio, &mut count_error, on_wifi_change_status),
                        Enabled         => Self::handle_enabled(&gpio, &mut count_error, on_wifi_change_status),
                        Connecting      => Self::handle_connecting(&gpio, &mut count_error, on_wifi_change_status),
                        WaitForIp       => Self::handle_wait_for_ip(&gpio, &mut link_status, on_wifi_change_status),
                        Connected       => Self::handle_connected(&gpio, &mut link_status, &mut rssi_old, led_on, on_wifi_change_status),
                        Disconnected    => Self::handle_disconnected(&gpio, on_wifi_change_status),
                        Error           => Self::handle_error(link_status, &mut count_error, on_wifi_change_status),
                        Resetting       => Self::handle_resetting(&gpio, on_wifi_change_status),
                    };

                    match ctrl {
                        WifiFsmControl::Continue => continue 'no_rtc,
                        WifiFsmControl::Break    => break Ok(Arc::new(())),
                        WifiFsmControl::Next     => {}
                    }

                    led_on = !led_on;
                    System::delay_with_to_tick(Duration::from_millis(100));
                }
            }
        });

        if let Err(e) = &ret {
            log_error!(APP_TAG, "Error spawning wifi thread: {:?}", e);
            self.thread_started.store(false, Ordering::Release);
        }

        self.thread = ret.unwrap();
    }
}

impl Wifi {

    pub fn set_config(
        ssid: Bytes<32>,
        password: Bytes<32>,
        auth: Auth,
        enabled: bool,
    ) {
        {
            let _lock = credentials_lock();
            unsafe {
                SSID = ssid;
                PASSWORD = password;
                AUTH = auth;
            }
        }
        ENABLED.store(enabled, Ordering::Release);
    }

    pub fn shared() -> Self {
        Self {
            handle: null_mut(),
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal),
            thread_started: AtomicBool::new(false),
        }
    }


        fn handle_disabled<const N: usize>(
        gpio: &Gpio<N>,
        count_error: &mut StackType,
        on_wifi_change_status: &'static dyn OnWifiChangeStatus,
    ) -> WifiFsmControl {
        use WifiStatus::*;
        gpio.write(&GpioPeripheral::Cyw43Led, 0);
        if !INITIALIZED.load(Ordering::Acquire) {
            log_info!(APP_TAG, "WIFI init");
            let _ = (WIFI_FN.init)(wifi_country!('I', 'T', 0));
            (WIFI_FN.enable_sta_mode)(null_mut());
            INITIALIZED.store(true, Ordering::Release);
        }
        System::delay_with_to_tick(Duration::from_millis(2_500));
        *count_error = 0;
        transition_wifi_status!(Enabled, on_wifi_change_status);
        on_wifi_change_status.on_rssi_change(RSSIStatus::Unknown);
        WifiFsmControl::Next
    }

    fn handle_enabled<'a, const N: usize>(
        gpio: &Gpio<N>,
        count_error: &mut StackType,
        on_wifi_change_status: &'a dyn OnWifiChangeStatus,
    ) -> WifiFsmControl {
        use WifiStatus::*;
        gpio.write(&GpioPeripheral::Cyw43Led, 0);
        (WIFI_FN.enable_sta_mode)(null_mut());
        *count_error = 0;
        transition_wifi_status!(Connecting, on_wifi_change_status);
        on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
        WifiFsmControl::Next
    }

    fn handle_connecting<'a, const N: usize>(
        gpio: &Gpio<N>,
        count_error: &mut StackType,
        on_wifi_change_status: &'a dyn OnWifiChangeStatus,
    ) -> WifiFsmControl {
        use WifiStatus::*;
        gpio.write(&GpioPeripheral::Cyw43Led, 0);
        // Copied out: `connect` takes seconds, `set_config` must not wait for it
        let (ssid, password, auth) = {
            let _lock = credentials_lock();
            unsafe { (*&raw const SSID, *&raw const PASSWORD, AUTH) }
        };
        let ret = (WIFI_FN.connect)(null_mut(), ssid.as_str(), password.as_str(), auth).unwrap_or(1);
        if ret != PICO_OK as i32 {
            log_error!(APP_TAG, "Error connecting to WiFi, errno: {ret}");
            transition_wifi_status!(Error, on_wifi_change_status);
            return WifiFsmControl::Continue;
        }
        *count_error = 0;
        transition_wifi_status!(WaitForIp, on_wifi_change_status);
        on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
        WifiFsmControl::Continue
    }

    fn handle_wait_for_ip<'a, const N: usize>(
        gpio: &Gpio<N>,
        link_status: &mut LinkStatus,
        on_wifi_change_status: &'a dyn OnWifiChangeStatus,
    ) -> WifiFsmControl {
        use WifiStatus::*;
        gpio.write(&GpioPeripheral::Cyw43Led, 0);
        {
            *link_status = (WIFI_FN.link_status)(null_mut());
            match *link_status {
                LinkStatus::Up => {
                    transition_wifi_status!(Connected, on_wifi_change_status);
                    on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                }
                LinkStatus::WaitForIp => log_debug!(APP_TAG, "WiFi connected, waiting for DHCP..."),
                LinkStatus::Down => {
                    transition_wifi_status!(Error, on_wifi_change_status);
                    on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                }
                LinkStatus::LinkNoNet => {
                    log_debug!(APP_TAG, "WiFi connected but no network connectivity");
                    transition_wifi_status!(Error, on_wifi_change_status);
                    on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                }
                LinkStatus::BadAuth => {
                    log_debug!(APP_TAG, "WiFi authentication failed");
                    transition_wifi_status!(Error, on_wifi_change_status);
                    on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                }
            }
        }
        WifiFsmControl::Next
    }

    fn handle_connected<'a, const N: usize>(
        gpio: &Gpio<N>,
        link_status: &mut LinkStatus,
        rssi_old: &mut i8,
        led_on: bool,
        on_wifi_change_status: &'a dyn OnWifiChangeStatus,
    ) -> WifiFsmControl {
        use WifiStatus::*;
        {
            *link_status = (WIFI_FN.link_status)(null_mut());
            match *link_status {
                LinkStatus::Up => {
                    gpio.write(&GpioPeripheral::Cyw43Led, led_on as u32);
                    let rssi = if let Ok(rssi) = (WIFI_FN.get_rssi)(null_mut()) {
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
                    if *rssi_old != rssi.into() {
                        on_wifi_change_status.on_rssi_change(rssi);
                        *rssi_old = rssi.into();
                    }
                    System::delay_with_to_tick(Duration::from_millis(500));
                }
                LinkStatus::WaitForIp => {
                    transition_wifi_status!(WaitForIp, on_wifi_change_status);
                }
                LinkStatus::Down => {
                    transition_wifi_status!(Error, on_wifi_change_status);
                    on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                }
                LinkStatus::LinkNoNet => {
                    log_debug!(APP_TAG, "WiFi connected but no network connectivity");
                    transition_wifi_status!(Error, on_wifi_change_status);
                    on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                }
                LinkStatus::BadAuth => {
                    log_debug!(APP_TAG, "WiFi authentication failed");
                    transition_wifi_status!(Error, on_wifi_change_status);
                    on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
                }
            }
        }
        WifiFsmControl::Next
    }

    fn handle_disconnected<'a, const N: usize>(
        gpio: &Gpio<N>,
        on_wifi_change_status: &'a dyn OnWifiChangeStatus,
    ) -> WifiFsmControl {
        (WIFI_FN.disable_sta_mode)(null_mut());
        let _ = (WIFI_FN.drop)(null_mut());
        INITIALIZED.store(false, Ordering::Release);
        System::delay_with_to_tick(Duration::from_secs(5));
        on_wifi_change_status.on_rssi_change(RSSIStatus::NoSignal);
        gpio.write(&GpioPeripheral::Cyw43Led, 0);
        WifiFsmControl::Break
    }

    fn handle_error<'a>(
        _: LinkStatus,
        count_error: &mut StackType,
        on_wifi_change_status: &'a dyn OnWifiChangeStatus,
    ) -> WifiFsmControl {
        use WifiStatus::*;
        if *count_error < MAX_ERROR {
            *count_error += 1;
            FSM_STATUS_CURRENT.store(FSM_STATUS_OLD.load(Ordering::Acquire), Ordering::Release);
            FSM_STATUS_OLD.store(Error as u8, Ordering::Release);
            log_error!(APP_TAG, "Error status_current:{status_current} status_old:{status_old} {count_error}/{MAX_ERROR} retry...", status_current = status_current(), status_old = status_old());
            System::delay_with_to_tick(Duration::from_millis(1_000));
        } else {
            FSM_STATUS_OLD.store(Error as u8, Ordering::Release);
            //FSM_STATUS_CURRENT = if link_status == LinkStatus::BadAuth { Disconnected } else { Resetting };
            FSM_STATUS_CURRENT.store(Resetting as u8, Ordering::Release);
            log_error!(APP_TAG, "Resetting status_current:{status_current} status_old:{status_old} WiFi after {MAX_ERROR} retry...", status_current = status_current(), status_old = status_old());
            let _ = on_wifi_change_status.on_status_change(Error, status_current());
        }
        WifiFsmControl::Next
    }

    fn handle_resetting<'a, const N: usize>(
        gpio: &Gpio<N>,
        on_wifi_change_status: &'a dyn OnWifiChangeStatus,
    ) -> WifiFsmControl {
        use WifiStatus::*;
        log_warning!(APP_TAG, "Resetting WiFi wait 5 seconds...");
        gpio.write(&GpioPeripheral::Cyw43Led, 0);
        (WIFI_FN.disable_sta_mode)(null_mut());
        let _ = (WIFI_FN.drop)(null_mut());
        INITIALIZED.store(false, Ordering::Release);
        System::delay_with_to_tick(Duration::from_millis(2_500));
        transition_wifi_status!(Disabled, on_wifi_change_status);
        WifiFsmControl::Next
    }

}
