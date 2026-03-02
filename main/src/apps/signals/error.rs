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


use core::fmt::Display;

use crate::define_signal;


define_signal!(ErrorSignal, ERROR_SIGNAL);

pub enum ErrorFlag {
    None = 0x00,
    NTP = 0x01,
    DateTime = 0x02,
    Display = 0x04,
    DisplayHeader = 0x08,

}

impl From<u32> for ErrorFlag {
    fn from(value: u32) -> Self {
        use ErrorFlag::*;
        match value {
            0x01 => NTP,
            0x02 => DateTime,
            0x04 => Display,
            _ => None, // Default case, can be adjusted as needed
        }
    }
}

impl From<ErrorFlag> for u32 {
    fn from(flag: ErrorFlag) -> Self {
        flag as u32
    }
}

impl Display for ErrorFlag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use ErrorFlag::*;
        let s = match self {
            None => "None",
            NTP => "NTP",
            DateTime => "DateTime",
            Display => "Display",
            DisplayHeader => "DisplayHeader",
        };
        write!(f, "{}", s)
    }
}

#[macro_export]
macro_rules! set_app_error {
    ($result:expr, $flag:expr) => {
        if let Err(e) = $result {
            use crate::traits::signal::Signal;
            osal_rs::log_error!("AppErrorSignal", "App error: {}: {}", $flag, e);
            $crate::apps::signals::error::ErrorSignal::set($flag.into());
        }
    };
}