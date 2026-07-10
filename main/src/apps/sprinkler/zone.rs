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

use core::fmt::{Display, Formatter};

use osal_rs::utils::Bytes;
use osal_rs_serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{apps::DISPLAY_INPUT_MAX_SIZE, drivers::platform::GpioPeripheral};
use super::commons::Status;
use ZoneRelay::*;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(in crate::apps) enum ZoneRelay {
    Relay1,
    Relay2,
    Relay3,
    Relay4,
}

impl From<ZoneRelay> for &str {
    fn from(value: ZoneRelay) -> Self {
        match value {
            Relay1 => "Relay 1",
            Relay2 => "Relay 2",
            Relay3 => "Relay 3",
            Relay4 => "Relay 4",
        }
    }
}

impl Display for ZoneRelay {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", <ZoneRelay as Into<&str>>::into(*self))
    }
}


impl From<u8> for ZoneRelay {
    fn from(value: u8) -> Self {
        match value {
            1 => Relay1,
            2 => Relay2,
            3 => Relay3,
            4 => Relay4,
            _ => Relay1
        }
    }
}

impl From<ZoneRelay> for u8 {
    fn from(value: ZoneRelay) -> Self {
        match value {
            Relay1 => 1,
            Relay2 => 2,
            Relay3 => 3,
            Relay4 => 4
        }
    }
}

impl From<ZoneRelay> for GpioPeripheral {
    fn from(value: ZoneRelay) -> Self {
        match value {
            Relay1 => GpioPeripheral::Relay1,
            Relay2 => GpioPeripheral::Relay2,
            Relay3 => GpioPeripheral::Relay3,
            Relay4 => GpioPeripheral::Relay4
        }
    }
}


impl Serialize for ZoneRelay {
    #[inline]
    fn serialize<S: Serializer>(&self, name: &str, serializer: &mut S) -> Result<(), S::Error> {
        Ok(serializer.serialize_u8(name, *self as u8)?)
    }
}

impl Deserialize for ZoneRelay {
    #[inline]
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> Result<Self, D::Error> {
        Ok(ZoneRelay::from(deserializer.deserialize_u8(name)?))
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct Zone {

    /// description of zone
    pub(in crate::apps) description: Bytes<DISPLAY_INPUT_MAX_SIZE>,

    /// relay number associated to the zone
    pub(in crate::apps) relay_number: ZoneRelay,

    /// watering time in minutes
    pub(in crate::apps) watering_time: u8,

    /// for manage order of execution lighter is first then weightier
    pub(in crate::apps) weight: u8,

    /// status of the zone
    pub(in crate::apps) status: Status
}

impl Zone {
    pub(in crate::apps) const SIZE: usize = 4;

    pub(in crate::apps) fn new(relay_number: ZoneRelay) -> Self {
        Self {
            description: Bytes::from_str(relay_number.into()),
            relay_number,
            watering_time: 0,
            weight: 0,
            status: Status::UNACTIVE
        }
    }
}

impl Display for Zone {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.description.as_str()
        )
    }
}