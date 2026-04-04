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

use alloc::sync::Arc;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::{log_info};
use osal_rs::utils::Result;

use crate::apps::config::Config;
use crate::apps::signals::display::DisplaySignal;
use crate::apps::signals::error::{ErrorFlag, ErrorSignal};
use crate::drivers::date_time::DateTime;
use crate::drivers::network::Network;
use crate::set_app_error;
use crate::traits::rtc::RTC;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::traits::wifi::{OnWifiChangeStatus, RSSIStatus, WifiStatus::{self, *}};


const APP_TAG: &str = "AppWifi";

static mut STATUS: WifiStatus = Disabled;

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

pub struct Wifi(Option<Arc<Mutex<dyn RTC + 'static>>>);

impl Initializable for Wifi {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app wifi");

        Ok(())
    }
}

impl OnWifiChangeStatus for Wifi {
    fn on_status_change(&self, _: WifiStatus, status: WifiStatus) -> Result<()>  {

        unsafe {
            STATUS = status;
        }

        let config = Config::shared();

        match status {
            Disabled | Enabled | Connecting | WaitForIp => log_info!(APP_TAG, "Waiting for IP status: {status:?}"),
            Connected => {
                log_info!(APP_TAG, "Connected ip: {}", Network::dhcp_get_ip_address());

                let timestamp = ntp_sync!(APP_TAG, config);
                

                if timestamp > 0 {

                    match &self.0 {
                        Some(rtc) => {
                            let dt = DateTime::from_timestamp_locale(timestamp, true);
                            set_app_error!(dt.clone(), ErrorFlag::NTP);
                            set_app_error!(rtc.lock().unwrap().set_timestamp(timestamp), ErrorFlag::NTP);
                            log_info!(APP_TAG, "NTP time: {}", dt.unwrap());
                        },
                        None => {
                            log_info!(APP_TAG, "RTC not set for WifiApp");
                            ErrorSignal::set(ErrorFlag::NTP.into());
                        }
                        
                    }
                }
                
            },
            Disconnected | Resetting => log_info!(APP_TAG, "Disconnected"),
            Error => log_info!(APP_TAG, "Error")
        };
        Ok(())
    }

    fn on_rssi_change(&self, rssi: RSSIStatus) {
        DisplaySignal::set((rssi.to_bites() as u32) << 6);
    }
}

impl Wifi {
    pub fn shared() -> Self {
        Self(None)
    }

    #[allow(dead_code)]
    #[inline]
    pub const fn get_status(&self) -> WifiStatus {
        unsafe { STATUS }
    }

    pub fn set_rtc(&mut self, rtc: Arc<Mutex<dyn RTC + 'static>>) {
        self.0 = Some(rtc);
    }
}
