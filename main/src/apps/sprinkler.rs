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
use cjson_binding::{from_json, to_json};
use osal_rs::os::Thread;
use osal_rs::os::types::StackType;
use osal_rs::{access_static_option, log_error, log_info, log_warning};
use osal_rs::utils::{Error, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::sprinkler::schedule::Schedule;
use crate::drivers::filesystem::{FileBytes, Filesystem, flags};
use crate::drivers::platform::{FS_CONFIG_DIR, FS_SEPARATOR_DIR};
use crate::traits::state::Initializable;

mod commons;
pub(in crate::apps) mod zone;
pub(in crate::apps) mod schedule;

const APP_TAG: &str = "AppSprinkler";
const MAX_SCHEDULES: usize = 4;

// static mut THREAD: Option<Thread> = None;
// const THREAD_NAME: &str = "app_sprinkler_trd";
// const STACK_SIZE: StackType = 1_024;
// const TICK_INTERVAL_MS: u16 = 100;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct Sprinkler {
    schedules: [Schedule; MAX_SCHEDULES]
}

#[derive(Clone, Copy)]
struct SprinklerPtr(usize);

impl Initializable for Sprinkler {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app config");
        
        self.load()?;

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

    pub(in crate::apps) fn new() -> Self {
        Self {
            schedules: [Schedule::default(); MAX_SCHEDULES]
        }
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

    pub(in crate::apps) fn start(&mut self) {
        log_info!(APP_TAG, "Starting Sprinkler app");

        // let thread = access_static_option!(THREAD);
        // let app_param = SprinklerPtr( (&raw const self) as usize ); 
    }
}