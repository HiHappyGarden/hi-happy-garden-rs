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

//! WiFi connectivity module for Hi Happy Garden

use cyw43::Control;
use defmt::*;

use crate::config::AppConfig;

/// WiFi connection error
#[derive(Debug, defmt::Format)]
pub enum WifiError {
    /// Not configured
    NotConfigured,
    /// Connection failed
    ConnectionFailed,
    /// Join failed
    JoinFailed,
}

/// Connect to WiFi using the provided configuration
pub async fn connect<'a>(
    control: &mut Control<'a>,
    config: &AppConfig,
) -> Result<(), WifiError> {
    if !config.is_wifi_configured() {
        warn!("WiFi not configured");
        return Err(WifiError::NotConfigured);
    }

    info!("Connecting to WiFi SSID: {}", config.wifi_ssid.as_str());

    // Join the WiFi network
    match control
        .join(config.wifi_ssid.as_str(), cyw43::JoinOptions::new(config.wifi_passwd.as_bytes()))
        .await
    {
        Ok(()) => {
            info!("Successfully joined WiFi network");
            Ok(())
        }
        Err(_e) => {
            warn!("Failed to join WiFi network");
            Err(WifiError::JoinFailed)
        }
    }
}

/// Disconnect from WiFi
pub async fn disconnect<'a>(control: &mut Control<'a>) {
    info!("Disconnecting from WiFi");
    control.leave().await;
}
