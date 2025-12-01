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

//! Irrigation control module for Hi Happy Garden

use defmt::*;
use embassy_rp::gpio::Output;

use crate::config::MAX_ZONES;

/// Default irrigation duration in seconds (5 minutes)
pub const DEFAULT_IRRIGATION_DURATION_SECS: u32 = 300;

/// Zone identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum ZoneId {
    Zone1 = 0,
    Zone2 = 1,
    Zone3 = 2,
    Zone4 = 3,
}

impl ZoneId {
    /// Convert from index to ZoneId
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(ZoneId::Zone1),
            1 => Some(ZoneId::Zone2),
            2 => Some(ZoneId::Zone3),
            3 => Some(ZoneId::Zone4),
            _ => None,
        }
    }
}

/// Zone state
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum ZoneState {
    /// Zone is off
    Off,
    /// Zone is on, with remaining time in seconds
    On { remaining_secs: u32 },
}

/// Single irrigation zone
pub struct Zone<'a> {
    /// Zone identifier
    pub id: ZoneId,
    /// Relay output pin
    pin: Output<'a>,
    /// Current state
    state: ZoneState,
}

impl<'a> Zone<'a> {
    /// Create a new zone
    pub fn new(id: ZoneId, pin: Output<'a>) -> Self {
        Self {
            id,
            pin,
            state: ZoneState::Off,
        }
    }

    /// Turn on the zone for a specified duration
    pub fn turn_on(&mut self, duration_secs: u32) {
        info!("Zone {:?}: turning ON for {} seconds", self.id, duration_secs);
        self.pin.set_high();
        self.state = ZoneState::On {
            remaining_secs: duration_secs,
        };
    }

    /// Turn off the zone
    pub fn turn_off(&mut self) {
        info!("Zone {:?}: turning OFF", self.id);
        self.pin.set_low();
        self.state = ZoneState::Off;
    }

    /// Check if zone is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, ZoneState::On { .. })
    }

    /// Get current state
    pub fn state(&self) -> ZoneState {
        self.state
    }

    /// Update the zone timer (call every second)
    pub fn update(&mut self) {
        if let ZoneState::On { remaining_secs } = &mut self.state {
            if *remaining_secs > 0 {
                *remaining_secs -= 1;
            }
            if *remaining_secs == 0 {
                self.turn_off();
            }
        }
    }
}

/// Irrigation controller managing multiple zones
pub struct IrrigationController<'a> {
    /// Array of zones
    zones: [Zone<'a>; MAX_ZONES],
}

impl<'a> IrrigationController<'a> {
    /// Create a new irrigation controller
    pub fn new(zones: [Zone<'a>; MAX_ZONES]) -> Self {
        Self { zones }
    }

    /// Start irrigation on a specific zone with default duration
    pub fn start_zone(&mut self, zone_id: ZoneId) {
        let index = zone_id as usize;
        if index < MAX_ZONES {
            self.zones[index].turn_on(DEFAULT_IRRIGATION_DURATION_SECS);
        }
    }

    /// Start irrigation with custom duration
    pub fn start_zone_with_duration(&mut self, zone_id: ZoneId, duration_secs: u32) {
        let index = zone_id as usize;
        if index < MAX_ZONES {
            self.zones[index].turn_on(duration_secs);
        }
    }

    /// Stop irrigation on a specific zone
    pub fn stop_zone(&mut self, zone_id: ZoneId) {
        let index = zone_id as usize;
        if index < MAX_ZONES {
            self.zones[index].turn_off();
        }
    }

    /// Stop all zones
    pub fn stop_all(&mut self) {
        for zone in &mut self.zones {
            zone.turn_off();
        }
    }

    /// Check if any zone is active
    pub fn is_any_active(&self) -> bool {
        self.zones.iter().any(|z| z.is_active())
    }

    /// Get the state of a specific zone
    pub fn get_zone_state(&self, zone_id: ZoneId) -> ZoneState {
        let index = zone_id as usize;
        if index < MAX_ZONES {
            self.zones[index].state()
        } else {
            ZoneState::Off
        }
    }

    /// Update all zone timers (call every second)
    pub fn update(&mut self) {
        for zone in &mut self.zones {
            zone.update();
        }
    }

    /// Check schedules and return the zone that should be activated
    /// For now, this is a placeholder - schedule checking will be implemented
    /// when RTC support is added
    pub fn check_schedules(&self) -> Option<ZoneId> {
        // TODO: Implement schedule checking with RTC
        None
    }
}
