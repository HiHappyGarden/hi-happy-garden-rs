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

use osal_rs::{log_debug, log_info, utils::Result};

use crate::traits::{state::Initializable, wifi::OnWifiChangeStatus};

const APP_TAG: &str = "AppWifi";

pub struct WifiApp;

impl Initializable for WifiApp {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app wifi");

        
        Ok(())
    }
}

impl OnWifiChangeStatus<'static> for WifiApp {
    fn on_status_change(&self, old_status: crate::traits::wifi::WifiStatus, status: crate::traits::wifi::WifiStatus) {
        log_debug!(APP_TAG, "FSM_STATUS_OLD: {} -> FSM_STATUS_CURRENT: {}", old_status, status);
    }
}

impl WifiApp {
    pub const fn new() -> Self {
        Self
    }
}