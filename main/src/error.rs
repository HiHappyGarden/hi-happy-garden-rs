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

use core::fmt;
use alloc::string::String;
use osal_rs::utils::Error as OsalError;

/// Custom error type for Hi Happy Garden application
#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    /// Hardware initialization or operation failed
    Hardware(String),
    /// Configuration error
    Config(String),
    /// Network/WiFi error
    Network(String),
    /// Display error
    Display(String),
    /// Filesystem error
    Filesystem(String),
    /// Sensor error (RTC, temperature, etc)
    Sensor(String),
    /// OSAL-RS error (wrapped)
    Osal(OsalError<'static>),
    /// Generic error with context
    Generic(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Hardware(msg) => write!(f, "Hardware error: {}", msg),
            AppError::Config(msg) => write!(f, "Config error: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Display(msg) => write!(f, "Display error: {}", msg),
            AppError::Filesystem(msg) => write!(f, "Filesystem error: {}", msg),
            AppError::Sensor(msg) => write!(f, "Sensor error: {}", msg),
            AppError::Osal(err) => write!(f, "OSAL error: {}", err),
            AppError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

/// Automatic conversion from OSAL errors
impl From<OsalError<'static>> for AppError {
    fn from(err: OsalError<'static>) -> Self {
        AppError::Osal(err)
    }
}

/// Result type alias for application operations
pub type AppResult<T> = core::result::Result<T, AppError>;

/// Macro to create hardware errors with context
#[macro_export]
macro_rules! hw_error {
    ($($arg:tt)*) => {
        $crate::error::AppError::Hardware(alloc::format!($($arg)*))
    };
}

/// Macro to create config errors with context
#[macro_export]
macro_rules! cfg_error {
    ($($arg:tt)*) => {
        $crate::error::AppError::Config(alloc::format!($($arg)*))
    };
}

/// Macro to create network errors with context
#[macro_export]
macro_rules! net_error {
    ($($arg:tt)*) => {
        $crate::error::AppError::Network(alloc::format!($($arg)*))
    };
}

