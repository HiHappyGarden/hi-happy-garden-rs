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

use osal_rs_serde::{Deserialize, Deserializer, Serialize, Serializer};

 #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub(in crate::apps) enum Status {
    #[default]
    UNACTIVE,
    ACTIVE,
    RUN
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        match value {
            0 => Status::UNACTIVE,
            1 => Status::ACTIVE,
            2 => Status::RUN,
            _ => Status::UNACTIVE
        }
    }
}

impl From<Status> for u8 {
    fn from(value: Status) -> Self {
        match value {
            Status::UNACTIVE => 0,
            Status::ACTIVE => 1,
            Status::RUN => 2,
        }
    }
}

impl Serialize for Status {
    #[inline]
    fn serialize<S: Serializer>(&self, name: &str, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_u8(name, (*self).into())?;
        Ok(())
    }
}

impl Deserialize for Status {
    #[inline]
    fn deserialize<D: Deserializer>(deserializer: &mut D, name: &str) -> Result<Self, D::Error> {
        Ok(Status::from(deserializer.deserialize_u8(name)?))
    }
}

