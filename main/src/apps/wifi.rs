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
use core::sync::atomic::{AtomicU8, Ordering};
use osal_rs::os::{Mutex, MutexFn, RawMutex, RawMutexGuard, System, SystemFn};
use osal_rs::os::types::EventBits;
use osal_rs::{log_info};
use osal_rs::utils::{Bytes, OnceBox, Result};

use crate::apps::config::Config;
use crate::apps::signals::display::DisplaySignal;
use crate::apps::signals::error::{ErrorFlag, ErrorSignal};
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::drivers::date_time::DateTime;
use crate::drivers::network::Network;
use crate::set_app_error;
use crate::traits::rtc::RTC;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::traits::wifi::{OnWifiChangeStatus, RSSIStatus, WifiStatus::{self, *}};
use crate::traits::network::IPV6_ADDR_LEN;

const APP_TAG: &str = "AppWifi";

/// `WifiStatus` as `u8`, set by the WiFi thread
static STATUS: AtomicU8 = AtomicU8::new(Disabled as u8);
/// Written by the WiFi thread, read by the display: only under `MUTEX`
static mut IP_ADDR: Bytes<IPV6_ADDR_LEN> = Bytes::new();
static MUTEX: OnceBox<RawMutex> = OnceBox::new();

#[inline]
fn ip_addr_lock() -> RawMutexGuard<RawMutex> {
    RawMutexGuard::acquire(MUTEX.get_or_try_init(RawMutex::new).expect("Failed to create app wifi mutex"))
}

macro_rules! ntp_sync {
    ($tag:expr, $config:expr) => {
        {
            log_info!($tag, "NTP Config: server={}, port={}, msg_len={}", 
                $config.get_ntp_config().get_server(), 
                $config.get_ntp_config().get_port(), 
                $config.get_ntp_config().get_msg_len());
            
            match Network::dns_resolve_addrress(&$config.get_ntp_config().get_server()) {
                Ok(ip) => {
                    log_info!($tag, "NTP Server IP: {}", ip);

                    match Network::ntp_request(ip, $config.get_ntp_config().get_port(), $config.get_ntp_config().get_msg_len()) {
                        Ok(timestamp) => {
                            log_info!($tag, "NTP request successful, timestamp: {}", timestamp);
                            timestamp
                        }
                        Err(e) => {
                            log_info!($tag, "NTP request failed: {}", e);
                            0
                        }
                    }
                },
                Err(_) => {
                    log_info!($tag, "Failed to resolve NTP server address");
                    0
                }
            }
        }
    };
}

pub(in crate::apps) struct Wifi(Option<Arc<Mutex<dyn RTC + 'static>>>);

impl Initializable for Wifi {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app wifi");

        Ok(())
    }
}

impl OnWifiChangeStatus for Wifi {
    fn on_status_change(&self, _: WifiStatus, status: WifiStatus) -> Result<()>  {

        STATUS.store(status as u8, Ordering::Release);

        let config = Config::shared();

        match status {
            Disabled | Enabled | Connecting | WaitForIp => {
                StatusSignal::clear(StatusFlag::WifiReady.into());
                log_info!(APP_TAG, "Waiting for IP status: {status:?}")
            }
            Connected => {
                let ip_addr = Network::dhcp_get_ip_address();
                {
                    let _lock = ip_addr_lock();
                    unsafe { IP_ADDR = ip_addr; }
                }
                log_info!(APP_TAG, "Connected ip: {}", ip_addr);
                
                StatusSignal::set(StatusFlag::WifiReady.into());
                StatusSignal::clear(StatusFlag::NtpError.into());

                let timestamp = ntp_sync!(APP_TAG, config);
                

                if timestamp > 0 {

                    match &self.0 {
                        Some(rtc) => {
                            let dt = DateTime::from_timestamp(timestamp);
                            set_app_error!(dt.clone(), ErrorFlag::NTP);
                            set_app_error!(rtc.lock().unwrap().set_timestamp(timestamp), ErrorFlag::NTP);
                            log_info!(APP_TAG, "NTP time (UTC): {}", dt.unwrap());
                            log_info!(APP_TAG, "NTP sync complete, heap_free:{}", System::get_free_heap_size());
                        },
                        None => {
                            log_info!(APP_TAG, "RTC not set for WifiApp");
                            ErrorSignal::set(ErrorFlag::NTP.into());
                            StatusSignal::set(StatusFlag::NtpError.into());                                
                        }
                        
                    }
                } else {
                    StatusSignal::set(StatusFlag::NtpError.into());
                }
                
            },
            Disconnected | Resetting => log_info!(APP_TAG, "Disconnected"),
            Error => log_info!(APP_TAG, "Error")
        };
        Ok(())
    }

    fn on_rssi_change(&self, rssi: RSSIStatus) {
        DisplaySignal::set((rssi.to_bites() as EventBits) << 6);
    }
}

impl Wifi {
    pub(in crate::apps) fn shared() -> Self {
        Self(None)
    }

    #[allow(dead_code)]
    #[inline]
    pub(in crate::apps) fn get_status(&self) -> WifiStatus {
        WifiStatus::from(STATUS.load(Ordering::Acquire))
    }

    pub(in crate::apps) fn set_rtc(&mut self, rtc: Arc<Mutex<dyn RTC + 'static>>) {
        self.0 = Some(rtc);
    }

    pub(in crate::apps) fn get_ip_address() -> Bytes<IPV6_ADDR_LEN> {
        let _lock = ip_addr_lock();
        unsafe { *&raw const IP_ADDR }
    }
}
