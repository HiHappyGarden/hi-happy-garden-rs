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

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use cjson_binding::{from_json, to_json};
use osal_rs::{log_error, log_info, log_warning};
use osal_rs::utils::{Error, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::sprinkler::commons::Status;
use crate::apps::sprinkler::schedule::Schedule;
use crate::apps::sprinkler::zone::{Zone, ZoneRelay};
use crate::drivers::date_time::DateTime;
use crate::drivers::filesystem::{FileBytes, Filesystem, flags};
use crate::drivers::platform::{FS_CONFIG_DIR, FS_SEPARATOR_DIR};
use crate::traits::state::Initializable;

mod commons;
pub(in crate::apps) mod zone;
pub(in crate::apps) mod schedule;

const APP_TAG: &str = "AppSprinkler";

static DISBURSEMENT_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Index of the schedule returned by `query`, set via `AT+SPK=select,<index>`
static SELECTED_SCHEDULE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct Sprinkler {
    schedules: [Schedule; Schedule::SIZE],
    zones: [Zone; Zone::SIZE]
}

impl Initializable for Sprinkler {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app sprinkler");
        
        self.reinit();

        self.load()?;

        Ok(())
    }
}

impl Default for Sprinkler {

    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Sprinkler {
    const FILE_NAME: &'static str = "sprinkler.json";
    pub(in crate::apps) const AT_CMD: &'static str = "AT+SPK";
    pub(in crate::apps) const AT_RESP: &'static str = "+SPK: ";

    pub(in crate::apps) const fn new() -> Self {
        Self {
            schedules: [Schedule::new(); Schedule::SIZE],
            zones: [Zone::new(ZoneRelay::Relay0), Zone::new(ZoneRelay::Relay1), Zone::new(ZoneRelay::Relay2), Zone::new(ZoneRelay::Relay3)]
        }
    }

    #[inline]
    fn reinit(&mut self) {
        self.schedules.iter_mut().for_each(| schedule | *schedule = Schedule::new());
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

            self.schedules = Sprinkler::new().schedules;

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
                self.schedules = Sprinkler::new().schedules;

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

}