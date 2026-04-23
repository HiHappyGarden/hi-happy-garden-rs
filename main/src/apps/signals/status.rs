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

use osal_rs::utils::Bytes;

///! FSM signal for status updates.

use crate::{define_signal, traits::rx_tx::Source};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusFlag {
    None = 0x00,
    Startup = 0x01,
    EnableSystemHandler = 0x02,
    EnableSession = 0x04,
    EnableParser = 0x08,
    EnableDisplay = 0x10,
    CheckConfig = 0x20,
    EnableWifi = 0x40,
    Ready = 0x80,
    Error = 0x01_00,
    Reset = 0x02_00,
    
    SystemCmd = 0x00_10_00_00,
    MqttCmd = 0x00_20_00_00,
    UartCmd = 0x00_40_00_00,
    UserLogged = 0x00_80_00_00,
}

impl From<u32> for StatusFlag {
    fn from(value: u32) -> Self {
        use StatusFlag::*;
        match value {
            0x00 => None,
            0x01 => Startup,
            0x02 => EnableSystemHandler,
            0x04 => EnableSession,
            0x08 => EnableParser,
            0x10 => EnableDisplay,
            0x20 => CheckConfig,
            0x40 => EnableWifi,
            0x80 => Ready,
            0x01_00 => Error,
            0x02_00 => Reset,
            
            0x00_10_00_00 => SystemCmd,
            0x00_20_00_00 => MqttCmd,
            0x00_40_00_00 => UartCmd, 
            0x00_80_00_00 => UserLogged,
            _ => None, // Default case, can be adjusted as needed
        }
    }
}

impl From<StatusFlag> for u32 {
    fn from(flag: StatusFlag) -> Self {
        flag as u32
    }
}

impl From<&Source> for StatusFlag {
    fn from(source: &Source) -> Self {
        use StatusFlag::*;
        match source {
            Source::System => SystemCmd,
            Source::Mqtt => MqttCmd,
            Source::Uart => UartCmd,
        }
    }
}

impl StatusFlag {
    fn as_str(&self) -> Bytes<24> {
        use StatusFlag::*;
        match self {
            None => Bytes::from("None"),
            Startup => Bytes::from("Startup"),
            EnableSystemHandler => Bytes::from("EnableSystemHandler"),
            EnableSession => Bytes::from("EnableSession"),
            EnableParser => Bytes::from("EnableParser"),
            EnableDisplay => Bytes::from("EnableDisplay"),
            CheckConfig => Bytes::from("CheckConfig"),
            EnableWifi => Bytes::from("EnableWifi"),
            Ready => Bytes::from("Ready"),
            Error => Bytes::from("Error"),
            Reset => Bytes::from("Reset"),
            SystemCmd => Bytes::from("SystemCmd"),
            MqttCmd => Bytes::from("MqttCmd"),
            UartCmd => Bytes::from("UartCmd"),
            UserLogged => Bytes::from("UserLogged"),
        }
    }
}

define_signal!(StatusSignal, STATUS_SIGNAL);