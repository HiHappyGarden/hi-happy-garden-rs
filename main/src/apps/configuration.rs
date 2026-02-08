/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/
#![allow(dead_code)]

use alloc::format;
use alloc::string::String;

use cjson_binding::{from_json, to_json};

use osal_rs::utils::Bytes;
use osal_rs::utils::{Result, Error};

use osal_rs_serde::{Deserialize, Serialize};

use crate::drivers::filesystem::{open_flags, FileBytes, Filesystem};
use crate::drivers::wifi::Auth;
use crate::drivers::platform::{FS_CONFIG_DIR, FS_SEPARATOR_DIR};


static mut WIFI_CONFIG: WifiConfig = WifiConfig {
    version: 0,
    ssid: Bytes::new(),
    password: Bytes::new(),
    hostname: Bytes::new(),
    enabled: false,
    auth: Auth::Wpa2
};

static mut CONFIG: Config = Config {
    version: 0,
    timezone: 0,
    daylight_saving_time: false,
    users: [UserConfig {
        version: 0,
        user: Bytes::new(),
        password: Bytes::new()
    }; 2]
};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct WifiConfig {
    version: u8,
    ssid: Bytes<32>,
    password: Bytes<64>,
    hostname: Bytes<32>,
    enabled: bool,
    auth: Auth
}

impl Default for WifiConfig {
    fn default() -> Self {
        Self {
            version: 0,
            ssid: Bytes::new(),
            password: Bytes::new(),
            hostname: Bytes::new(),
            enabled: false,
            auth: Auth::Wpa2
        }
    }
}

impl WifiConfig {

    const FILE_NAME: &'static str = "wifi_config.json";


    pub fn load() -> Result<()> {
        let mut file_name = FileBytes::new_by_str(FS_CONFIG_DIR);
        file_name.append_str(FS_SEPARATOR_DIR);
        file_name.append_str(WifiConfig::FILE_NAME);

        let mut file = Filesystem::open_with_as_sync_str(&file_name, open_flags::RDONLY | open_flags::CREAT)?;
        let wifi_json = file.read_with_as_sync_str(true)?;
        let wifi_json = match String::from_utf8(wifi_json) {
            Ok(json) => json,
            Err(e) => {
                return Err(Error::UnhandledOwned(format!("Failed to parse wifi config JSON: {}", e)));
            }
        };
 
        from_json::<WifiConfig>(&wifi_json)
            .map_err(|e| Error::UnhandledOwned(format!("Failed to deserialize wifi config JSON: {}", e)))
            .and_then(|config| {
                unsafe {
                    WIFI_CONFIG = config;
                }
                Ok(())
            })
    }

    pub fn save(_config: Self)  -> Result<()> {
        let mut file_name = FileBytes::new_by_str(FS_CONFIG_DIR);
        file_name.append_str(FS_SEPARATOR_DIR);
        file_name.append_str(WifiConfig::FILE_NAME);

        unsafe {
            to_json(&*&raw const WIFI_CONFIG)
                .map_err(|e| Error::UnhandledOwned(format!("Failed to serialize wifi config to JSON: {}", e)))
                .and_then(|json| {
                    let json_bytes = json.into_bytes();
                    
                    let mut file = Filesystem::open_with_as_sync_str(&file_name, open_flags::WRONLY | open_flags::CREAT)?;
                    file.write(&json_bytes, true)?;

                    Ok(())
                })
        }
    }

    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn get_ssid(&self) -> &Bytes<32> {
        &self.ssid
    }

    pub fn get_password(&self) -> &Bytes<64> {
        &self.password
    }

    pub fn get_hostname(&self) -> &Bytes<32> {
        &self.hostname
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_auth(&self) -> Auth {
        self.auth
    }

    pub fn set_ssid(&mut self, ssid: Bytes<32>) {
        self.ssid = ssid;
    }

    pub fn set_password(&mut self, password: Bytes<64>) {
        self.password = password;
    }

    pub fn set_hostname(&mut self, hostname: Bytes<32>) {
        self.hostname = hostname;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_auth(&mut self, auth: Auth) {
        self.auth = auth;
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct UserConfig {
    version: u8,
    user: Bytes<32>,
    password: Bytes<64>
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            version: 0,
            user: Bytes::new(),
            password: Bytes::new()
        }
    }
}

impl UserConfig {
    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn get_user(&self) -> &Bytes<32> {
        &self.user
    }

    pub fn get_password(&self) -> &Bytes<64> {
        &self.password
    }

    pub fn set_user(&mut self, user: Bytes<32>) {
        self.user = user;
    }

    pub fn set_password(&mut self, password: Bytes<64>) {
        self.password = password;
    }
    
}


#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    version: u8,
    timezone: u16,
    daylight_saving_time: bool,
    users: [UserConfig; 2],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 0,
            timezone: 0,
            daylight_saving_time: false,
            users: [Default::default(); 2],
        }
    }
}

impl Config {

    const FILE_NAME: &'static str = "config.json";

    pub fn load() -> Result<()> {
        let mut file_name = FileBytes::new_by_str(FS_CONFIG_DIR);
        file_name.append_str(FS_SEPARATOR_DIR);
        file_name.append_str(Config::FILE_NAME);

        let mut file = Filesystem::open_with_as_sync_str(&file_name, open_flags::RDONLY | open_flags::CREAT)?;
        let wifi_json = file.read_with_as_sync_str(true)?;
        let wifi_json = match String::from_utf8(wifi_json) {
            Ok(json) => json,
            Err(e) => {
                return Err(Error::UnhandledOwned(format!("Failed to parse config JSON: {}", e)));
            }
        };
 
        from_json::<Config>(&wifi_json)
            .map_err(|e| Error::UnhandledOwned(format!("Failed to deserialize config JSON: {}", e)))
            .and_then(|config| {
                unsafe {
                    CONFIG = config;
                }
                Ok(())
            })
    }

    pub fn save(_config: Self)  -> Result<()> {
        let mut file_name = FileBytes::new_by_str(FS_CONFIG_DIR);
        file_name.append_str(FS_SEPARATOR_DIR);
        file_name.append_str(Config::FILE_NAME);

        unsafe {
            to_json(&*&raw const CONFIG)
                .map_err(|e| Error::UnhandledOwned(format!("Failed to serialize config to JSON: {}", e)))
                .and_then(|json| {
                    let json_bytes = json.into_bytes();
                    
                    let mut file = Filesystem::open_with_as_sync_str(&file_name, open_flags::WRONLY | open_flags::CREAT)?;
                    file.write(&json_bytes, true)?;

                    Ok(())
                })
        }
    }

    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn get_timezone(&self) -> u16 {
        self.timezone
    }

    pub fn is_daylight_saving_time(&self) -> bool {
        self.daylight_saving_time
    }

    pub fn get_users(&self) -> &[UserConfig; 2] {
        &self.users
    }

    pub fn set_timezone(&mut self, timezone: u16) {
        self.timezone = timezone;
    }

    pub fn set_daylight_saving_time(&mut self, dst: bool) {
        self.daylight_saving_time = dst;
    }

    pub fn set_user(&mut self, index: usize, user: UserConfig) {
        if index < self.users.len() {
            self.users[index] = user;
        }
    }
}