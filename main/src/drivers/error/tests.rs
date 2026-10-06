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


//! Board health after `Hardware::init`, plus the hardware error flags.
//!
//! `test_no_hardware_error` is the key check of the whole driver suite:
//! every peripheral init that fails sets its bit in `HardwareErrorSignal`
//! instead of stopping the boot, so a zero there means the board came up
//! complete.

use alloc::format;

use osal_rs::log_debug;
use osal_rs::os::types::EventBits;
use osal_rs::utils::Result;

use super::{HardwareErrorFlag, HardwareErrorSignal};
use crate::drivers::platform::Hardware;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::hardware::HardwareFn;
use crate::traits::signal::Signal;

const TAG: &str = "HardwareErrorTests";

const ALL_FLAGS: [HardwareErrorFlag; 12] = [
    HardwareErrorFlag::Filesystem,
    HardwareErrorFlag::Button,
    HardwareErrorFlag::Encoder,
    HardwareErrorFlag::Gpio,
    HardwareErrorFlag::I2C,
    HardwareErrorFlag::Display,
    HardwareErrorFlag::Network,
    HardwareErrorFlag::Relays,
    HardwareErrorFlag::Leds,
    HardwareErrorFlag::Rtc,
    HardwareErrorFlag::Uart,
    HardwareErrorFlag::Wifi,
];

fn test_no_hardware_error() -> Result<()> {
    let errors = HardwareErrorSignal::get();
    for flag in ALL_FLAGS {
        if errors & EventBits::from(flag) != 0 {
            log_debug!(TAG, "set: {flag}");
        }
    }
    test_assert_eq!(errors, 0);
    Ok(())
}

fn test_flags_roundtrip() -> Result<()> {
    let mut mask: EventBits = 0;
    for flag in ALL_FLAGS {
        let bits = EventBits::from(flag);
        test_assert_eq!(bits.count_ones(), 1);
        test_assert!(mask & bits == 0, "{flag} overlaps another flag");
        mask |= bits;
        test_assert_eq!(EventBits::from(HardwareErrorFlag::from(bits)), bits);
        test_assert!(!format!("{flag}").is_empty());
    }
    Ok(())
}

fn test_flags_try_from_invalid() -> Result<()> {
    // Bit combinations come straight from HardwareErrorSignal::get(): they
    // must be rejected, not panic (open_bugs #10)
    let all = ALL_FLAGS.iter().fold(0 as EventBits, |mask, &flag| mask | EventBits::from(flag));
    for value in [0, 0x03, 0x21, all, 0x1000, all + 1, EventBits::MAX] {
        test_assert_eq!(HardwareErrorFlag::from(value), HardwareErrorFlag::None, "0x{value:x} accepted");
    }
    Ok(())
}

fn test_signal_set_clear() -> Result<()> {
    // Rtc is not set on a healthy board (checked above), so it can be borrowed
    let flag = EventBits::from(HardwareErrorFlag::Rtc);
    let before = HardwareErrorSignal::get();

    HardwareErrorSignal::set(flag);
    let set = HardwareErrorSignal::get();
    HardwareErrorSignal::clear(flag);
    let cleared = HardwareErrorSignal::get();

    test_assert!(set & flag != 0);
    test_assert_eq!(cleared, before & !flag);
    Ok(())
}

fn test_unique_id() -> Result<()> {
    let id = Hardware::get_unique_id();
    log_debug!(TAG, "unique id: {:02x?}", id);
    test_assert!(id != [0u8; 8] && id != [0xffu8; 8], "unique id not programmed: {id:02x?}");
    // Read from OTP: never changes, the filesystem AES key derives from it
    test_assert_eq!(Hardware::get_unique_id(), id);
    Ok(())
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_no_hardware_error,
        test_flags_roundtrip,
        test_flags_try_from_invalid,
        test_signal_set_clear,
        test_unique_id,
    );
}
