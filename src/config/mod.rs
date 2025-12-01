/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2025 Antonio Salsi <passy.linux@zresa.it>
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

//! Configuration management module for Hi Happy Garden

use heapless::String;

/// Maximum length for WiFi SSID
pub const MAX_SSID_LEN: usize = 32;

/// Maximum length for WiFi password
pub const MAX_PASSWD_LEN: usize = 64;

/// Minimum length for WiFi password (WPA2 requirement)
pub const MIN_PASSWD_LEN: usize = 8;

/// Maximum number of zones supported
pub const MAX_ZONES: usize = 4;

/// Maximum number of schedules per zone
pub const MAX_SCHEDULES_PER_ZONE: usize = 4;

/// Application configuration
#[derive(Clone)]
pub struct AppConfig {
    /// WiFi SSID
    pub wifi_ssid: String<MAX_SSID_LEN>,
    /// WiFi password
    pub wifi_passwd: String<MAX_PASSWD_LEN>,
    /// WiFi enabled flag
    pub wifi_enabled: bool,
    /// Timezone offset in hours
    pub timezone: i8,
    /// Daylight saving time enabled
    pub daylight_saving: bool,
    /// Zone configurations
    pub zones: [ZoneConfig; MAX_ZONES],
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            wifi_ssid: String::new(),
            wifi_passwd: String::new(),
            wifi_enabled: false,
            timezone: 0,
            daylight_saving: false,
            zones: [
                ZoneConfig::default(),
                ZoneConfig::default(),
                ZoneConfig::default(),
                ZoneConfig::default(),
            ],
        }
    }
}

impl AppConfig {
    /// Create a new configuration with WiFi credentials
    pub fn with_wifi(ssid: &str, passwd: &str) -> Self {
        let mut config = Self::default();
        let _ = config.wifi_ssid.push_str(ssid);
        let _ = config.wifi_passwd.push_str(passwd);
        config.wifi_enabled = true;
        config
    }

    /// Check if WiFi is configured with valid credentials
    pub fn is_wifi_configured(&self) -> bool {
        !self.wifi_ssid.is_empty() 
            && self.wifi_passwd.len() >= MIN_PASSWD_LEN
    }
}

/// Configuration for a single irrigation zone
#[derive(Clone, Default)]
pub struct ZoneConfig {
    /// Zone name
    pub name: String<16>,
    /// Zone enabled flag
    pub enabled: bool,
    /// Schedules for this zone
    pub schedules: [Schedule; MAX_SCHEDULES_PER_ZONE],
}

/// Irrigation schedule
#[derive(Clone, Copy, Default)]
pub struct Schedule {
    /// Hour to start (0-23)
    pub start_hour: u8,
    /// Minute to start (0-59)
    pub start_minute: u8,
    /// Duration in minutes
    pub duration_minutes: u16,
    /// Days of week (bit 0 = Sunday, bit 6 = Saturday)
    pub days_of_week: u8,
    /// Schedule enabled flag
    pub enabled: bool,
}

impl Schedule {
    /// Create a new schedule
    pub const fn new(
        start_hour: u8,
        start_minute: u8,
        duration_minutes: u16,
        days_of_week: u8,
    ) -> Self {
        Self {
            start_hour,
            start_minute,
            duration_minutes,
            days_of_week,
            enabled: true,
        }
    }

    /// Check if schedule is active for a given day
    pub const fn is_active_on_day(&self, day: u8) -> bool {
        self.enabled && (self.days_of_week & (1 << day)) != 0
    }
}
