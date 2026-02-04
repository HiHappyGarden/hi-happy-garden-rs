
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

use core::fmt::Display;

pub enum WifiStatus {
    Disabled,
    Enabling,
    Enabled,
    Connecting,
    Connected,
    Disconnecting,
    Error
}

impl Display for WifiStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use WifiStatus::*;
        match self {
            Disabled => write!(f, "Disabled"),
            Enabling => write!(f, "Enabling"),
            Enabled => write!(f, "Enabled"),
            Connecting => write!(f, "Connecting"),
            Connected => write!(f, "Connected"),
            Disconnecting => write!(f, "Disconnecting"),
            Error => write!(f, "Error"),
        }
    }
}

pub trait OnWifiChangeStatus: Send + Sync {

    fn on_status_change(&self, status: WifiStatus, old_status: WifiStatus);
    
}