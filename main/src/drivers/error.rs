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

#![allow(dead_code)]

use core::fmt::Error;
use core::fmt::{Display, Formatter};
use osal_rs::utils::Result;

use crate::define_signal;

const APP_TAG: &str = "HardwareErrorSignal";

define_signal!(HardwareErrorSignal, HARDWARE_ERROR_SIGNAL);

#[derive(Debug, Clone, Copy)]
pub enum HardwareErrorFlag {
    Filesystem = 0x01,
    Button = 0x02,
    Encoder = 0x04,
    Gpio = 0x08,
    I2C = 0x10,
    Display = 0x20,
    Network = 0x40,
    Relays = 0x80,
    Leds = 0x100,
    Rtc = 0x200,
    Uart = 0x400,
    Wifi = 0x800,
}

impl From<u32> for HardwareErrorFlag {
    fn from(value: u32) -> Self {
        use HardwareErrorFlag::*;
        match value {
            0x01 => Filesystem,
            0x02 => Button,
            0x04 => Encoder,
            0x08 => Gpio,
            0x10 => I2C,
            0x20 => Display,
            0x40 => Network,
            0x80 => Relays,
            0x100 => Leds,
            0x200 => Rtc,
            0x400 => Uart,
            0x800 => Wifi,
            _ => panic!("Invalid hardware flag value: {}", value),
        }
    }
}

impl From<HardwareErrorFlag> for u32 {
    fn from(flag: HardwareErrorFlag) -> Self {
        flag as u32
    }
}

impl Display for HardwareErrorFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use HardwareErrorFlag::*;
        let description = match self {
            Filesystem => "Filesystem error",
            Button => "Button error",
            Encoder => "Encoder error",
            Gpio => "GPIO error",
            I2C => "I2C error",
            Display => "Display error",
            Network => "Network error",
            Relays => "Relays error",
            Leds => "LEDs error",
            Rtc => "RTC error",
            Uart => "UART error",
            Wifi => "WiFi error",
        };
        write!(f, "{}", description)
    }
}

#[macro_export]
macro_rules! set_hardware_error {
    ($result:expr, $flag:expr) => {
        if let Err(e) = $result {
            use crate::traits::signal::Signal;
            osal_rs::log_error!("HardwareErrorSignal", "Hardware error: {}: {}", $flag, e);
            $crate::drivers::error::HardwareErrorSignal::set($flag.into());
        }
    };
}