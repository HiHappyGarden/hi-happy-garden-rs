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


use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use cjson_binding::{from_json, to_json};
use osal_rs::{access_static_option, log_error, log_info, log_warning};
use osal_rs::os::{RawMutex, RawMutexGuard};
use osal_rs::utils::{Error, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::drivers::filesystem::flags::{CREAT, RDWR, TRUNC, WRONLY};
use crate::drivers::filesystem::{FileBytes, Filesystem};
use crate::drivers::platform::FS_SEPARATOR_DIR;



pub(in crate::apps) fn deserialize_file<T>(mutex: &'static Option<RawMutex>, app_tag: &str, dir: &str, name: &str) -> Result<T> 
where 
    T: Deserialize + Serialize + Default
{
    let _lock = RawMutexGuard::acquire(access_static_option!(mutex));

    let mut file_name = FileBytes::from_str(dir);
    file_name.append_str(FS_SEPARATOR_DIR);
    file_name.append_str(name);

    let mut file = match Filesystem::open_with_as_sync_str(
        &file_name,
        RDWR | CREAT,
    ) {
        Ok(file) => file,
        Err(e @ Error::ReturnWithCode(-2)) => {
            log_warning!(app_tag, "Failed to open file:{file_name} - {e}, try to create it");
            Filesystem::open_with_as_sync_str(
                &file_name,
                WRONLY | CREAT,
            )?
        }
        Err(e) => return Err(e),
    };

    let json = match file.read_with_as_sync_str(true) {
        Ok(json) => json,
        Err(e) => {
            log_error!(app_tag, "Failed to read file:{file_name}, using defaults: {e}");
            Vec::new()
        }
    };
    drop(file);

    // If file is empty or doesn't exist, use defaults
    if json.is_empty() {
        log_warning!(app_tag, "File:{file_name} not found or empty, using defaults");

        let ret = T::default();


        serialize_file(mutex, app_tag, dir, name, &ret)?;

        return Ok(ret);
    }

    let json = match String::from_utf8(json) {
        Ok(json) => json,
        Err(e) => {
            return Err(Error::UnhandledOwned(format!(
                "Failed to parse config JSON: {e}"
            )));
        }
    };


    match from_json::<T>(&json) {
        Ok(t) => {

            log_info!(app_tag, "File:{file_name} loaded successfully");
            log_info!(app_tag, "{json}");

            Ok(t)
        }
        Err(e) => {
            log_warning!(app_tag, "Using default config values err: {e}");
            let ret = T::default();

            serialize_file(mutex, app_tag, dir, name, &ret)?;

            Ok(ret)
        }
    }
}

pub(in crate::apps) fn serialize_file<'a, T>(mutex: &'static Option<RawMutex>, app_tag: &str, dir: &str, name: &str, t: &'a T) -> Result<&'a T> 
where 
    T: Deserialize + Serialize + Default
{
    let _lock = RawMutexGuard::acquire(access_static_option!(mutex));

    let mut file_name = FileBytes::from_str(dir);
    file_name.append_str(FS_SEPARATOR_DIR);
    file_name.append_str(name);

    to_json(t)
        .map_err(|e| {
            Error::UnhandledOwned(format!("Failed to serialize file:{file_name} to JSON: {e}"))
        })
        .and_then(|json| {
            let json_bytes = json.into_bytes();

            let mut file = match Filesystem::open_with_as_sync_str(
                &file_name,
                WRONLY | CREAT | TRUNC,
            ) {
                Ok(file) => file,
                Err(e @ Error::ReturnWithCode(-2)) => {
                    log_warning!(app_tag, "Failed to open file:{file_name} - {e}, try to create it");
                    Filesystem::open_with_as_sync_str(
                        &file_name,
                        WRONLY | CREAT | TRUNC,
                    )?
                }
                Err(e) => return Err(e),
            };

            file.write(&json_bytes, true)?;

            log_info!(app_tag, "Saved successfully");
            Ok(t)
        })
}