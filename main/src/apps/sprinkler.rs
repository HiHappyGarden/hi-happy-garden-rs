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


use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use cjson_binding::from_json;
use osal_rs::{log_error, log_info, log_warning};
use osal_rs::utils::{Error, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::sprinkler::schedule::Schedule;
use crate::drivers::filesystem::{FileBytes, Filesystem, flags};
use crate::drivers::platform::{FS_CONFIG_DIR, FS_SEPARATOR_DIR};
use crate::traits::state::Initializable;

mod commons;
pub(super) mod zone;
pub(super) mod schedule;

const APP_TAG: &str = "AppSprinkler";
const MAX_SCHEDULES: usize = 4;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(super) struct Sprinkler {

    pub schedules: [Schedule; MAX_SCHEDULES]
}

impl Initializable for Sprinkler {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app config");
        

        Ok(())
    }
}

impl Default for Sprinkler {
    fn default() -> Self {
        Self {
            schedules: [Schedule::default(); MAX_SCHEDULES]
        }
    }
}

impl Sprinkler {
    const FILE_NAME: &'static str = "sprinkler.json";
    pub(in crate::apps) const AT_CMD: &'static str = "AT+CNF";
    pub(in crate::apps) const AT_RESP: &'static str = "+CNF: ";

    pub(super) fn new() -> Self {
        Self {
            schedules: [Schedule::default(); MAX_SCHEDULES]
        }
    }

    pub(super) fn load(&mut self) -> Result<()> {
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

        // If file is empty or doesn't exist, use defaults
        if json.is_empty() {
            log_info!(APP_TAG, "Sprinkler file not found or empty, using defaults");

            self.schedules = Sprinkler::default().schedules;

            Self::save()?;

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

                Self::save()?;

                Ok(())
            }
        }
    }

    pub(super) fn save() -> Result<()> {
        log_info!(APP_TAG, "Save app config");
        Ok(())
    }

}