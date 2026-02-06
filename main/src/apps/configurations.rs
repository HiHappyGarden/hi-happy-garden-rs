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
    // auth: Auth::Wpa2
};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct WifiConfig {
    pub ssid: Bytes<32>,
    pub password: Bytes<64>,
    pub hostname: Bytes<32>,
    pub enabled: bool,
    // pub auth: Auth
}

// impl WifiConfig {
//
//     pub const fn load() {
//
//     }
//
//     pub const fn save(config: Self) {
//
//     }
//     pub fn get() -> Self {
//         unsafe { WIFI_CONFIG }.clone()
//     }
//
//     pub fn set(config: Self) {
//         //unsafe { WIFI_CONFIG = config };
//     }
// }