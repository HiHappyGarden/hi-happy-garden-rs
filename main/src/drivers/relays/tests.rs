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

//! Relay tests: every relay is really switched.
//!
//! Each relay is closed for [`PULSE_MS`] only, so with the valves connected
//! a few drops may come out; the click is the audible proof the output
//! works. All relays are always left open, pass or fail.

use core::time::Duration;

use osal_rs::os::System;
use osal_rs::utils::{OsalRsBool, Result};

use super::Relays;
use crate::drivers::platform::GpioPeripheral;
use crate::tests::{TestStats, hardware, run_tests, test_assert, test_assert_eq};
use crate::traits::relays::Relays as RelaysFn;

const TAG: &str = "RelaysTests";

const PULSE_MS: u64 = 300;

const RELAYS: [GpioPeripheral; 4] = [
    GpioPeripheral::Relay0,
    GpioPeripheral::Relay1,
    GpioPeripheral::Relay2,
    GpioPeripheral::Relay3,
];

fn pulse(relays: &dyn RelaysFn, relay: GpioPeripheral) -> Result<()> {
    let on = relays.set_relay_state(relay, true);
    System::delay_with_to_tick(Duration::from_millis(PULSE_MS));
    let off = relays.set_relay_state(relay, false);
    test_assert_eq!(on, OsalRsBool::True);
    test_assert_eq!(off, OsalRsBool::True);
    Ok(())
}

fn test_pulse_each_relay() -> Result<()> {
    let relays = Relays::shared();
    let ret = RELAYS.iter().try_for_each(|relay| pulse(&relays, *relay));
    relays.turn_off_all_relays();
    ret
}

fn test_pulse_through_hardware() -> Result<()> {
    // The path used by the application: Hardware -> Relays
    let hardware = hardware();
    let ret = RELAYS.iter().try_for_each(|relay| pulse(hardware, *relay));
    hardware.turn_off_all_relays();
    ret
}

fn test_reject_non_relay() -> Result<()> {
    let relays = Relays::shared();
    for peripheral in [GpioPeripheral::LedRed, GpioPeripheral::Btn, GpioPeripheral::InternalTemp, GpioPeripheral::NoUsed] {
        test_assert_eq!(relays.set_relay_state(peripheral, true), OsalRsBool::False);
    }
    Ok(())
}

fn test_outputs_not_readable() -> Result<()> {
    // Gpio::read only serves inputs: a relay state cannot be read back
    let relays = Relays::shared();
    test_assert!(relays.0.read(&GpioPeripheral::Relay0).is_err());
    Ok(())
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_reject_non_relay,
        test_outputs_not_readable,
        test_pulse_each_relay,
        test_pulse_through_hardware,
    );
}
