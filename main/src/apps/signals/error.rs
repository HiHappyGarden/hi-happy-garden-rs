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


use crate::define_signal;


define_signal!(ErrorSignal, ERROR_SIGNAL);

pub enum DisplayFlag {
    None = 0x00,
    DisplayError = 0x01,

}

impl From<u32> for DisplayFlag {
    fn from(value: u32) -> Self {
        use DisplayFlag::*;
        match value {
            0x01 => DisplayError,
            _ => None, // Default case, can be adjusted as needed
        }
    }
}

impl From<DisplayFlag> for u32 {
    fn from(flag: DisplayFlag) -> Self {
        flag as u32
    }
}