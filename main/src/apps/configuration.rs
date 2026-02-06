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

use osal_rs::utils::Bytes;
use osal_rs_serde::{Deserialize, Serialize};
use crate::drivers::wifi::Auth;

static mut WIFI_CONFIG: WifiConfig = WifiConfig {
    ssid: Bytes::new(),
    password: Bytes::new(),
    hostname: Bytes::new(),
    enabled: false,
    auth: Auth::Wpa2
};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct WifiConfig {
    ssid: Bytes<32>,
    password: Bytes<64>,
    hostname: Bytes<32>,
    enabled: bool,
    auth: Auth
}

impl WifiConfig {

    pub const fn load() {

    }

    pub const fn save(config: Self) {

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