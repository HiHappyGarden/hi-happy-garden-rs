
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
  
#![allow(dead_code)]

use core::fmt::{Debug, Display};

use osal_rs::utils::{Error, Result};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WifiStatus {
    Disabled = 0x00,
    Enabled = 0x01,
    Connecting = 0x02,
    WaitForIp = 0x04,
    Connected = 0x08,
    Disconnected = 0x10,
    Error = 0x20,
    Resetting = 0x40,
}

impl From<u8> for WifiStatus {
    fn from(value: u8) -> Self {
        use WifiStatus::*;
        match value {
            0x00 => Disabled,
            0x01 => Enabled,
            0x02 => Connecting,
            0x04 => WaitForIp,
            0x08 => Connected,
            0x10 => Disconnected,
            0x20 => Error,
            0x40 => Resetting,
            _ => Disabled,
        }
    }
}

impl Display for WifiStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use WifiStatus::*;
        match self {
            Disabled => write!(f, "Disabled"),
            Enabled => write!(f, "Enabled"),
            Connecting => write!(f, "Connecting"),
            WaitForIp => write!(f, "WaitForIp"),
            Connected => write!(f, "Connected"),
            Disconnected => write!(f, "Disconnected"),
            Error => write!(f, "Error"),
            Resetting => write!(f, "Resetting"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RSSIStatus {
    Unknown = 0,
    Excellent = -50,
    Good = -60,
    Fair = -70,
    Weak = -80,
    NoSignal = -90,
}

impl From<i8> for RSSIStatus {
    fn from(rssi: i8) -> Self {
        use RSSIStatus::*;
        match rssi {
            r if r >= -50 => Excellent,
            r if r >= -60 => Good,
            r if r >= -70 => Fair,
            r if r >= -80 => Weak,
            _ => NoSignal,
        }
    }
}

impl From<RSSIStatus> for i8 {
    fn from(status: RSSIStatus) -> Self {
        use RSSIStatus::*;
        match status {
            Unknown => 0,
            Excellent => -50,
            Good => -60,
            Fair => -70,
            Weak => -80,
            NoSignal => -90,
        }
    }
}

impl RSSIStatus {
    pub fn to_bites(&self) -> u8 {
        use RSSIStatus::*;
        match self {
            Unknown => 0x01,
            Excellent => 0x02,
            Good => 0x03,
            Fair => 0x04,
            Weak => 0x05,
            NoSignal => 0x06,
        }
    }

    pub fn from_bites(value: u8) -> Result<Self> {
        use RSSIStatus::*;
        match value {
            0x01 => Ok(Unknown),
            0x02 => Ok(Excellent),
            0x03 => Ok(Good),
            0x04 => Ok(Fair),
            0x05 => Ok(Weak),
            0x06 => Ok(NoSignal),
            _ => Err(Error::ReturnWithCode(value as i32)),
        }
    }
}

 pub trait SetOnWifiChangeStatus<'a> {
     fn set_on_wifi_change_status(&mut self, on_wifi_change_status: &'a dyn OnWifiChangeStatus);
 }

pub trait OnWifiChangeStatus: Send + Sync {

    fn on_status_change(&self, old_status: WifiStatus, status: WifiStatus) -> Result<()>;

    fn on_rssi_change(&self, rssi: RSSIStatus);
    
}