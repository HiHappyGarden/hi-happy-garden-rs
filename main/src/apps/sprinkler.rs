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



use core::sync::atomic::{AtomicBool, Ordering};

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use at_parser_rs::{AtError, AtResult};
use at_parser_rs::context::AtContext;
use cjson_binding::{from_json, to_json};
use osal_rs::{log_error, log_info, log_warning};
use osal_rs::utils::{Error, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::parser::{CMD_SIZE, NOT_LOGGED_RESPONSE, at_cmd_response};
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::sprinkler::commons::Status;
use crate::apps::sprinkler::schedule::{Schedule, ZONES_SIZE};
use crate::apps::sprinkler::zone::Zone;
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::drivers::date_time::DateTime;
use crate::drivers::filesystem::{FileBytes, Filesystem, flags};
use crate::drivers::platform::{FS_CONFIG_DIR, FS_SEPARATOR_DIR};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use osal_rs::utils::Bytes;

mod commons;
pub(in crate::apps) mod zone;
pub(in crate::apps) mod schedule;

const APP_TAG: &str = "AppSprinkler";
const MAX_SCHEDULES: usize = 4;

static DISBURSEMENT_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Index of the schedule returned by `query`, set via `AT+SPK=select,<index>`
static SELECTED_SCHEDULE: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct Sprinkler {
    schedules: [Schedule; MAX_SCHEDULES]
}

impl Initializable for Sprinkler {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app sprinkler");
        
        self.load()?;

        Ok(())
    }
}

impl Default for Sprinkler {
    fn default() -> Self {
        Self {
            schedules: [Schedule::default(); MAX_SCHEDULES],
        }
    }
}

impl AtContext<{ CMD_SIZE }> for Sprinkler {
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, { CMD_SIZE }> {
        if StatusSignal::get() & <StatusFlag as Into<u32>>::into(StatusFlag::UserLogged) == 0 {
            return Err((at_response, AtError::Unhandled(NOT_LOGGED_RESPONSE)));
        }

        let index = SELECTED_SCHEDULE.load(Ordering::Relaxed);
        let schedule = self.schedules.get(index)
            .ok_or((at_response, AtError::Unhandled("selected schedule index out of bounds")))?;

        let mut response = Bytes::<{ CMD_SIZE }>::new();
        use core::fmt::Write;
        let _ = write!(response, "{index}:{},{},{},{},{}\r\n",
            schedule.minute, schedule.hour, schedule.days, schedule.month,
            <Status as Into<u8>>::into(schedule.status));

        for (zone_index, zone) in schedule.zones.iter().enumerate() {
            let _ = write!(response, "{zone_index}:{},{},{},{}\r\n",
                zone.relay_number, zone.watering_time, zone.weight,
                <Status as Into<u8>>::into(zone.status));
        }

        Ok((at_response, response))
    }

    #[inline]
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, { CMD_SIZE }> {
        Ok(at_cmd_response!(at_response; "select,<index> | schedule,insert,<index>,[minute],[hour],[days],[month],[description] | schedule,delete,<index> | zone,<schedule_index>,insert,<index>,<watering_time>,[weight],[description] | zone,<schedule_index>,delete,<zone_index> | save"))
    }

    fn set(&mut self, at_response: &'static str, args: at_parser_rs::Args) -> AtResult<'_, { CMD_SIZE }> {
        if StatusSignal::get() & <StatusFlag as Into<u32>>::into(StatusFlag::UserLogged) == 0 {
            return Err((at_response, AtError::Unhandled(NOT_LOGGED_RESPONSE)));
        }

        let cmd = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
        match cmd.as_ref() {
            "select" => {
                let index: usize = args.get(1).ok_or((at_response, AtError::InvalidArgs))?
                    .parse().map_err(|_| (at_response, AtError::InvalidArgs))?;
                if index >= self.schedules.len() {
                    return Err((at_response, AtError::Unhandled("schedule index out of bounds")));
                }
                SELECTED_SCHEDULE.store(index, Ordering::Relaxed);
            }
            "schedule" => {
                let op = args.get(1).ok_or((at_response, AtError::InvalidArgs))?;
                match op.as_ref() {
                    "insert" => self.insert_schedule(at_response, &args)?,
                    "delete" => self.delete_schedule(at_response, &args)?,
                    _ => return Err((at_response, AtError::InvalidArgs)),
                }
            }
            "zone" => {
                let schedule_index: usize = args.get(1).ok_or((at_response, AtError::InvalidArgs))?
                    .parse().map_err(|_| (at_response, AtError::InvalidArgs))?;
                let schedule = self.schedules.get_mut(schedule_index)
                    .ok_or((at_response, AtError::Unhandled("schedule index out of bounds")))?;

                let op = args.get(2).ok_or((at_response, AtError::InvalidArgs))?;
                match op.as_ref() {
                    "insert" => insert_zone(at_response, schedule, &args)?,
                    "delete" => delete_zone(at_response, schedule, &args)?,
                    _ => return Err((at_response, AtError::InvalidArgs)),
                }
            }
            "save" => {
                self.save().map_err(|_| (at_response, AtError::Unhandled("Save error")))?;
            }
            _ => return Err((at_response, AtError::InvalidArgs)),
        }

        Ok(at_cmd_response!(at_response; ""))
    }
}

impl Sprinkler {
    const FILE_NAME: &'static str = "sprinkler.json";
    pub(in crate::apps) const AT_CMD: &'static str = "AT+SPK";
    pub(in crate::apps) const AT_RESP: &'static str = "+SPK: ";

    #[inline]
    pub(in crate::apps) fn new() -> Self {
        Self::default()
    }

    pub(in crate::apps) fn load(&mut self) -> Result<()> {
        let mut file_name = FileBytes::from_str(FS_CONFIG_DIR);
        file_name.append_str(FS_SEPARATOR_DIR);
        file_name.append_str(Sprinkler::FILE_NAME);

        let mut file = match Filesystem::open_with_as_sync_str(
            &file_name,
            flags::RDWR | flags::CREAT,
        ) {
            Ok(file) => file,
            Err(e @ Error::ReturnWithCode(-2)) => {
                log_warning!(APP_TAG, "Failed to open sprinkler file: {e}, try to create it");
                Filesystem::open_with_as_sync_str(
                    &file_name,
                    flags::WRONLY | flags::CREAT,
                )?
            }
            Err(e) => return Err(e),
        };

        let json = match file.read_with_as_sync_str(true) {
            Ok(json) => json,
            Err(e) => {
                log_error!(APP_TAG, "Failed to read sprinkler file, using defaults: {e}");
                Vec::new()
            }
        };
        drop(file);

        // If file is empty or doesn't exist, use defaults
        if json.is_empty() {
            log_warning!(APP_TAG, "Sprinkler file not found or empty, using defaults");

            self.schedules = Sprinkler::default().schedules;

            self.save()?;

            return Ok(());
        }

        let json = match String::from_utf8(json) {
            Ok(json) => json,
            Err(e) => {
                return Err(Error::UnhandledOwned(format!(
                    "Failed to parse sprinkler JSON: {e}"
                )));
            }
        };

        match from_json::<Sprinkler>(&json) {
            Ok(config) => {
                self.schedules = config.schedules;

                log_info!(APP_TAG, "Sprinkler loaded successfully");
                log_info!(APP_TAG, "{json}");

                Ok(())
            }
            Err(e) => {
                log_warning!(APP_TAG, "Using default config values err: {e}");
                self.schedules = Sprinkler::default().schedules;

                self.save()?;

                Ok(())
            }
        }
    }

    pub(in crate::apps) fn save(&self) -> Result<()> {
        let mut file_name = FileBytes::from_str(FS_CONFIG_DIR);
        file_name.append_str(FS_SEPARATOR_DIR);
        file_name.append_str(Sprinkler::FILE_NAME);

   
        to_json(self)
        .map_err(|e| {
            Error::UnhandledOwned(format!("Failed to serialize config to JSON: {e}"))
        })
        .and_then(|json| {
            let json_bytes = json.into_bytes();

            let mut file = match Filesystem::open_with_as_sync_str(
                &file_name,
                flags::WRONLY | flags::CREAT | flags::TRUNC,
            ) {
                Ok(file) => file,
                Err(e @ Error::ReturnWithCode(-2)) => {
                    log_warning!(APP_TAG, "Failed to open config file: {e}, try to create it");
                    Filesystem::open_with_as_sync_str(
                        &file_name,
                        flags::WRONLY | flags::CREAT | flags::TRUNC,
                    )?
                }
                Err(e) => return Err(e),
            };

            file.write(&json_bytes, true)?;

            log_info!(APP_TAG, "Config saved successfully");
            Ok(())
        })
        
    }

    pub(in crate::apps) fn check(&mut self, now: DateTime) {
        if DISBURSEMENT_IN_PROGRESS.load(Ordering::Relaxed) {
            return;
        }


        for schedule in self.schedules.iter_mut() {
            if schedule.executable(&now) {
                DISBURSEMENT_IN_PROGRESS.store(true, Ordering::Relaxed);
                schedule.status = Status::RUN;
                for _zone in schedule.zones.iter() {

                }
                break;
            }
        }
    }

    fn insert_schedule(&mut self, at_response: &'static str, args: &at_parser_rs::Args) -> core::result::Result<(), (&'static str, AtError<'static>)> {
        let index: usize = args.get(2).ok_or((at_response, AtError::InvalidArgs))?
            .parse().map_err(|_| (at_response, AtError::InvalidArgs))?;

        let slot = self.schedules.get_mut(index)
            .ok_or((at_response, AtError::Unhandled("schedule index out of bounds")))?;
        if slot.status != Status::UNACTIVE {
            return Err((at_response, AtError::Unhandled("schedule index already occupied")));
        }

        let minute: u8 = match args.get(3).filter(|arg| !arg.is_empty()) {
            Some(arg) => arg.parse().map_err(|_| (at_response, AtError::InvalidArgs))?,
            None => Schedule::NOT_SET,
        };
        let hour: u8 = match args.get(4).filter(|arg| !arg.is_empty()) {
            Some(arg) => arg.parse().map_err(|_| (at_response, AtError::InvalidArgs))?,
            None => Schedule::NOT_SET,
        };
        let days: u8 = match args.get(5).filter(|arg| !arg.is_empty()) {
            Some(arg) => arg.parse().map_err(|_| (at_response, AtError::InvalidArgs))?,
            None => Schedule::NOT_SET,
        };
        let month: u16 = match args.get(6).filter(|arg| !arg.is_empty()) {
            Some(arg) => arg.parse().map_err(|_| (at_response, AtError::InvalidArgs))?,
            None => Schedule::NOT_SET as u16,
        };
        let description = args.get(7).unwrap_or_default();
        if description.len() > DISPLAY_INPUT_MAX_SIZE {
            return Err((at_response, AtError::Unhandled("description too long")));
        }

        slot.minute = minute;
        slot.hour = hour;
        slot.days = days;
        slot.month = month;
        slot.description = Bytes::from_str(description.as_ref());
        slot.zones = [Zone::default(); ZONES_SIZE];
        slot.status = Status::ACTIVE;

        Ok(())
    }

    fn delete_schedule(&mut self, at_response: &'static str, args: &at_parser_rs::Args) -> core::result::Result<(), (&'static str, AtError<'static>)> {
        let index: usize = args.get(2).ok_or((at_response, AtError::InvalidArgs))?
            .parse().map_err(|_| (at_response, AtError::InvalidArgs))?;

        let schedule = self.schedules.get_mut(index)
            .ok_or((at_response, AtError::Unhandled("schedule index out of bounds")))?;

        *schedule = Schedule::default();

        Ok(())
    }
}

fn insert_zone(at_response: &'static str, schedule: &mut Schedule, args: &at_parser_rs::Args) -> core::result::Result<(), (&'static str, AtError<'static>)> {
    let index: usize = args.get(3).ok_or((at_response, AtError::InvalidArgs))?
        .parse().map_err(|_| (at_response, AtError::InvalidArgs))?;

    let slot = schedule.zones.get_mut(index)
        .ok_or((at_response, AtError::Unhandled("zone index out of bounds")))?;
    if slot.status != Status::UNACTIVE {
        return Err((at_response, AtError::Unhandled("zone index already occupied")));
    }

    let relay_number = u8::try_from(index).map_err(|_| (at_response, AtError::InvalidArgs))?;
    let watering_time: u8 = args.get(4).ok_or((at_response, AtError::InvalidArgs))?
        .parse().map_err(|_| (at_response, AtError::InvalidArgs))?;
    let weight: u8 = match args.get(5).filter(|arg| !arg.is_empty()) {
        Some(arg) => arg.parse().map_err(|_| (at_response, AtError::InvalidArgs))?,
        None => 0,
    };
    let description = args.get(6).unwrap_or_default();
    if description.len() > DISPLAY_INPUT_MAX_SIZE {
        return Err((at_response, AtError::Unhandled("description too long")));
    }

    *slot = Zone {
        description: Bytes::from_str(description.as_ref()),
        relay_number,
        watering_time,
        weight,
        status: Status::ACTIVE,
    };

    Ok(())
}

fn delete_zone(at_response: &'static str, schedule: &mut Schedule, args: &at_parser_rs::Args) -> core::result::Result<(), (&'static str, AtError<'static>)> {
    let index: usize = args.get(3).ok_or((at_response, AtError::InvalidArgs))?
        .parse().map_err(|_| (at_response, AtError::InvalidArgs))?;

    let zone = schedule.zones.get_mut(index)
        .ok_or((at_response, AtError::Unhandled("zone index out of bounds")))?;

    *zone = Zone::default();

    Ok(())
}