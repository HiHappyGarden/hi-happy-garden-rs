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

use core::fmt::{Display, Formatter};

use at_parser_rs::at_quoted as quoted;
use at_parser_rs::context::AtContext;
use at_parser_rs::{Args, AtError, AtResult};
use osal_rs::{access_static_option, log_info};
use osal_rs::os::RawMutex;
use osal_rs::os::RawMutexGuard;
use osal_rs::utils::{Bytes, Result};
use osal_rs_serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::parser::{Parser, at_cmd_response};
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::utils::deserialize_file;
use crate::drivers::platform::{FS_CONFIG_DIR, GpioPeripheral};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use super::commons::Status;
use ZoneRelay::*;

static mut SHARED: ZoneController = ZoneController { zones: [
    Zone::new(Relay0),
    Zone::new(Relay1),
    Zone::new(Relay2),
    Zone::new(Relay3)
]};

static mut MUTEX: Option<RawMutex> = None;

/// Temporary zone data used to stage changes from `set` until `exec` persists them
static mut ZONE_TMP: Zone = Zone::new(Relay0);

const APP_TAG: &str = "ZoneController";



#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub(in crate::apps) enum ZoneRelay {
    #[default]
    Relay0,
    Relay1,
    Relay2,
    Relay3,
}

impl From<ZoneRelay> for &str {
    fn from(value: ZoneRelay) -> Self {
        match value {
            Relay0 => "Relay 0",
            Relay1 => "Relay 1",
            Relay2 => "Relay 2",
            Relay3 => "Relay 3",
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
            0 => Relay0,
            1 => Relay1,
            2 => Relay2,
            3 => Relay3,
            _ => Relay0
        }
    }
}

impl From<ZoneRelay> for u8 {
    fn from(value: ZoneRelay) -> Self {
        match value {
            Relay0 => 0,
            Relay1 => 1,
            Relay2 => 2,
            Relay3 => 3
        }
    }
}

impl From<ZoneRelay> for GpioPeripheral {
    fn from(value: ZoneRelay) -> Self {
        match value {
            Relay0 => GpioPeripheral::Relay0,
            Relay1 => GpioPeripheral::Relay1,
            Relay2 => GpioPeripheral::Relay2,
            Relay3 => GpioPeripheral::Relay3
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

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct Zone {

    /// description of zone
    pub(in crate::apps) description: Bytes<DISPLAY_INPUT_MAX_SIZE>,

    /// relay number associated to the zone
    pub(in crate::apps) zone_relay: ZoneRelay,

    /// for manage order of execution lighter is first then weightier
    pub(in crate::apps) weight: u8,

    /// status of the zone
    pub(in crate::apps) status: Status
}

impl Display for Zone {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.description.as_str()
        )
    }
}


impl Zone {
    const fn new(zone_relay: ZoneRelay) -> Self {
        Self {
            description: Bytes::new(),
            zone_relay,
            weight: 0,
            status: Status::UNACTIVE
        }
    } 
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct ZoneController {
    zones: [Zone; ZoneController::SIZE]
}


impl Initializable for ZoneController {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init Zone");

        let _lock = RawMutexGuard::acquire(access_static_option!(MUTEX));

        unsafe {
            for Zone{description, weight, status, zone_relay, ..} in &mut *&raw mut SHARED.zones {
                description.push((*zone_relay).into())?;
                *weight = (*zone_relay).into();
                *status = Status::UNACTIVE;
            }
        }
        
        
        *self = deserialize_file::<ZoneController>(unsafe { &*&raw const MUTEX }, APP_TAG, FS_CONFIG_DIR, ZoneController::FILE_NAME)?;

        Ok(())
    }
}

impl AtContext<{Parser::CMD_SIZE}> for ZoneController {
    fn exec(&mut self, at_response: &'static str) -> AtResult<'_, {Parser::CMD_SIZE}> {
        let _lock = RawMutexGuard::acquire(access_static_option!(MUTEX));

        if StatusSignal::get() & <StatusFlag as Into<u32>>::into(StatusFlag::UserLogged) == 0 {
            return Err((at_response, AtError::Unhandled(Parser::NOT_LOGGED_RESPONSE)));
        }

        let Zone{zone_relay, description, weight, ..} = unsafe { &mut *&raw mut ZONE_TMP };

        let zone = self.zones.iter_mut().find(|zone| zone.zone_relay == *zone_relay)
            .ok_or((at_response, AtError::InvalidArgs))?;
        zone.weight = *weight;
        zone.description = *description;

        unsafe {
            ZONE_TMP = Zone::new(Relay0);
        }

        Ok(at_cmd_response!(at_response; ""))
    }

    fn query(&mut self, at_response: &'static str) -> AtResult<'_, {Parser::CMD_SIZE}> {
        let _lock = RawMutexGuard::acquire(access_static_option!(MUTEX));
        if StatusSignal::get() & <StatusFlag as Into<u32>>::into(StatusFlag::UserLogged) == 0 {
            return Err((at_response, AtError::Unhandled(Parser::NOT_LOGGED_RESPONSE)));
        }

        let mut response = Bytes::<{Parser::CMD_SIZE}>::new();
        for zone in self.zones.iter() {
            response.format(format_args!("{},{},{}\r\n",
                <ZoneRelay as Into<u8>>::into(zone.zone_relay), zone.weight, quoted!(zone.description.as_str())));
        }

        Ok((at_response, response))
    }

    #[inline]
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, {Parser::CMD_SIZE}> {
        Ok(at_cmd_response!(at_response; "<zone_relay>,weight,<value> | <zone_relay>,description,<value>"))
    }

    fn set(&mut self, at_response: &'static str, args: Args) -> AtResult<'_, {Parser::CMD_SIZE}> {
        if StatusSignal::get() & <StatusFlag as Into<u32>>::into(StatusFlag::UserLogged) == 0 {
            return Err((at_response, AtError::Unhandled(Parser::NOT_LOGGED_RESPONSE)));
        }

        let zone_relay: u8 = args.get(0).ok_or((at_response, AtError::InvalidArgs))?
            .parse().map_err(|_| (at_response, AtError::InvalidArgs))?;
        let zone_relay = ZoneRelay::from(zone_relay);
        let cmd = args.get(1).ok_or((at_response, AtError::InvalidArgs))?;

        let _lock = RawMutexGuard::acquire(access_static_option!(MUTEX));

        let zone = self.zones.iter().find(|zone| zone.zone_relay == zone_relay)
            .ok_or((at_response, AtError::InvalidArgs))?;

        unsafe {
            ZONE_TMP = *zone;
        }

        match cmd.as_ref() {
            "weight" => {
                let value: u8 = args.get(2).ok_or((at_response, AtError::InvalidArgs))?
                    .parse().map_err(|_| (at_response, AtError::InvalidArgs))?;
                unsafe {
                    ZONE_TMP.weight = value;
                }
            }
            "description" => {
                let value = args.get(2).ok_or((at_response, AtError::InvalidArgs))?;
                if value.len() > DISPLAY_INPUT_MAX_SIZE {
                    return Err((at_response, AtError::Unhandled("description max len exceeded")));
                }
                unsafe {
                    ZONE_TMP.description = Bytes::from_str(value.as_ref());
                }
            }
            _ => return Err((at_response, AtError::InvalidArgs)),
        }

        Ok(at_cmd_response!(at_response; ""))
    }
}



impl ZoneController {
    pub(in crate::apps) const SIZE: usize = 4;
    pub(in crate::apps) const AT_CMD: &'static str = "AT+ZN";
    pub(in crate::apps) const AT_RESP: &'static str = "+ZN: ";
    const FILE_NAME: &'static str = "zones.json";

    pub(in crate::apps) fn shared() -> &'static mut Self {
        unsafe {
            if (*&raw const MUTEX).is_none() {
                MUTEX = match RawMutex::new() {
                    Ok(mutex) => Some(mutex),
                    Err(_) =>  panic!("MUTEX is not initialized",),
                }
            }
        }

        let _lock = RawMutexGuard::acquire(access_static_option!(MUTEX));
        unsafe { &mut *&raw mut SHARED }
    }

}
