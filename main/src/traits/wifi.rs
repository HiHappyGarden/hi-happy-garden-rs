
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

use core::fmt::{Debug, Display};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WifiStatus {
    Disabled,
    Enabled,
    Connecting,
    WaitForIp,
    Connected,
    Disconnected,
    Error,
    Resetting,
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

 pub trait SetOnWifiChangeStatus<'a> {
     fn set_on_wifi_change_status(&mut self, on_wifi_change_status: &'a dyn OnWifiChangeStatus<'a>);
 }

pub trait OnWifiChangeStatus<'a>: Send + Sync {

    fn on_status_change(&self, old_status: WifiStatus, status: WifiStatus);
    
}