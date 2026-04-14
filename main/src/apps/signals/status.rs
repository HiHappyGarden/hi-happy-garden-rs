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

///! FSM signal for status updates.

use crate::{define_signal, traits::rx_tx::Source};


pub enum StatusFlag {
    None = 0x00,
    DisplayCmd = 0x00_10_00_00,
    MqttCmd = 0x00_20_00_00,
    UartCmd = 0x00_40_00_00,
    UserLogged = 0x00_80_00_00,
}

impl From<u32> for StatusFlag {
    fn from(value: u32) -> Self {
        use StatusFlag::*;
        match value {
            0x00 => None,
            0x00_10_00_00 => DisplayCmd,
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
        match source {
            Source::Display => StatusFlag::DisplayCmd,
            Source::Mqtt => StatusFlag::MqttCmd,
            Source::Uart => StatusFlag::UartCmd,
        }
    }
}


define_signal!(StatusSignal, STATUS_SIGNAL);