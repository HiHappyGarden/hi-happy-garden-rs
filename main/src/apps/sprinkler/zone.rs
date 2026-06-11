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

use osal_rs::utils::Bytes;
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use super::commons::Status;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct Zone {

    /// description of zone
    pub description: Bytes<DISPLAY_INPUT_MAX_SIZE>,

    /// relay number associated to the zone
    pub relay_number: u8,

    /// watering time in minutes
    pub watering_time: u8,

    /// for manage order of execution lighter is first then weightier
    pub weight: u8,

    /// status of the zone
    pub status: Status
}

impl Default for Zone {
    fn default() -> Self {
        Self {
            description: Bytes::new(),
            relay_number: 0,
            watering_time: 0,
            weight: 0,
            status: Status::UNACTIVE
        }
    }
}