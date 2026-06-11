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

use crate::apps::{DISPLAY_INPUT_MAX_SIZE, sprinkler::zone::Zone};
use super::commons::Status;

pub(in crate::apps) const NOT_SET: u8 = 0;
const ZONES_SIZE: usize = 4;

 #[allow(dead_code)]

pub(in crate::apps) enum Day {
    Sunday = 0x01,
    Monday = 0x02,
    Tuesday = 0x04,
    Wednesday = 0x08,
    Thursday = 0x10,
    Friday = 0x20,
    Saturday = 0x40
} 

 #[allow(dead_code)]
pub(in crate::apps) enum Month {
    January = 0x01,
    February = 0x02,
    March = 0x04,
    April = 0x08,
    May = 0x10,
    June = 0x20,
    July = 0x40,
    August = 0x80,
    September = 0x0100,
    October = 0x0200,
    November = 0x0400,
    December = 0x0800,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(in crate::apps) struct Schedule {

    ///  minute, values allowed 0 - 59
    pub minute: u8,

    /// hour, values allowed 0 - 23 or NOT_SET (0xFF) for every hour
    pub hour: u8,

    /// day of week from 0x01 to 0x40 or NOT_SET (0xFF) for every day, otherwise bitmask of Day
    pub days: u8,

    /// month, values allowed 0x01 to 0x0800 or NOT_SET (0xFFFF) for every month
    pub month: u16,

    /// description 
    pub description: Bytes<DISPLAY_INPUT_MAX_SIZE>,

    /// zones associated to the schedule
    pub zones: [Zone; ZONES_SIZE],

    /// status of the schedule
    pub status: Status
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            minute: 0,
            hour: 0,
            days: NOT_SET,
            month: NOT_SET as u16,
            description: Bytes::new(),
            zones: [Zone::default(); ZONES_SIZE],
            status: Status::UNACTIVE
        }
    }
}

// impl Schedule {

//     pub fn is_active(&self) -> bool {
//         matches!(self.status, Status::ACTIVE)
//     }

//     pub fn is_run(&self) -> bool {
//         matches!(self.status, Status::RUN)
//     }
// }