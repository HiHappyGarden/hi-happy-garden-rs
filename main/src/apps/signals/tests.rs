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


//! Signal tests: flag encodings and the FreeRTOS event groups behind
//! `StatusSignal` / `ErrorSignal`.
//!
//! Only bits no running task owns during the tests are toggled, and each
//! test puts them back.

use core::time::Duration;

use alloc::collections::BTreeSet;
use alloc::format;

use osal_rs::os::ToTick;
use osal_rs::os::{System, SystemFn};
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Error, Result};

use super::display::DisplayFlag;
use super::error::{ErrorFlag, ErrorSignal};
use super::status::{StatusFlag, StatusSignal};
use crate::set_app_error;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::rx_tx::Source;
use crate::traits::signal::Signal;

const TAG: &str = "SignalsTests";

const STATUS_FLAGS: [StatusFlag; 17] = [
    StatusFlag::None,
    StatusFlag::Startup,
    StatusFlag::EnableSystemHandler,
    StatusFlag::EnableSession,
    StatusFlag::EnableParser,
    StatusFlag::EnableDisplay,
    StatusFlag::CheckConfig,
    StatusFlag::EnableWifi,
    StatusFlag::Ready,
    StatusFlag::WifiReady,
    StatusFlag::Error,
    StatusFlag::Reset,
    StatusFlag::NtpError,
    StatusFlag::SystemCmd,
    StatusFlag::MqttCmd,
    StatusFlag::UartCmd,
    StatusFlag::UserLogged,
];

const ERROR_FLAGS: [ErrorFlag; 5] = [
    ErrorFlag::None,
    ErrorFlag::NTP,
    ErrorFlag::DateTime,
    ErrorFlag::Display,
    ErrorFlag::DisplayHeader,
];

fn test_status_flags() -> Result<()> {
    let mut names = BTreeSet::new();
    let mut mask: EventBits = 0;
    for flag in STATUS_FLAGS {
        let bits = EventBits::from(flag);
        test_assert_eq!(StatusFlag::from(bits), flag);
        test_assert!(mask & bits == 0, "{flag:?} overlaps another flag");
        mask |= bits;
        test_assert!(names.insert(format!("{}", flag.as_str().as_str())), "duplicated name {flag:?}");
    }
    test_assert_eq!(StatusFlag::from(0x8000_0000), StatusFlag::None);
    test_assert_eq!(StatusFlag::from(&Source::Uart), StatusFlag::UartCmd);
    test_assert_eq!(StatusFlag::from(&Source::Mqtt), StatusFlag::MqttCmd);
    Ok(())
}

fn test_status_check_signal() -> Result<()> {
    let signal = EventBits::from(StatusFlag::Ready) | EventBits::from(StatusFlag::UserLogged);
    test_assert!(StatusFlag::Ready.check_signal(signal));
    test_assert!(StatusFlag::UserLogged.check_signal(signal));
    test_assert!(!StatusFlag::WifiReady.check_signal(signal));
    Ok(())
}

fn test_error_flags() -> Result<()> {
    for flag in ERROR_FLAGS {
        test_assert_eq!(ErrorFlag::from(EventBits::from(flag)), flag);
        test_assert!(!format!("{flag}").is_empty());
    }
    Ok(())
}

fn test_display_flags() -> Result<()> {
    // The wifi quality values share bits on purpose (a 3-bit field), they
    // only have to decode back to themselves
    let flags = [
        DisplayFlag::ButtonPressed,
        DisplayFlag::ButtonReleased,
        DisplayFlag::EncoderButtonPressed,
        DisplayFlag::EncoderButtonReleased,
        DisplayFlag::EncoderRotatedClockwise,
        DisplayFlag::EncoderRotatedCounterClockwise,
        DisplayFlag::WifiStatusUnknown,
        DisplayFlag::WifiStatusExcellent,
        DisplayFlag::WifiStatusGood,
        DisplayFlag::WifiStatusFair,
        DisplayFlag::WifiStatusWeak,
        DisplayFlag::WifiStatusNoSignal,
        DisplayFlag::Draw,
    ];
    for flag in flags {
        test_assert_eq!(DisplayFlag::from(EventBits::from(flag)), flag);
    }
    Ok(())
}

fn test_status_signal_set_clear() -> Result<()> {
    let bit = EventBits::from(StatusFlag::Reset);
    let before = StatusSignal::get();
    test_assert!(before & bit == 0);

    StatusSignal::set(bit);
    let set = StatusSignal::get();
    StatusSignal::clear(bit);
    let cleared = StatusSignal::get();

    test_assert_eq!(set, before | bit);
    test_assert_eq!(cleared, before);
    Ok(())
}

fn test_status_signal_wait() -> Result<()> {
    let bit = EventBits::from(StatusFlag::Reset);

    StatusSignal::set(bit);
    let start = System::get_tick_count();
    let bits = StatusSignal::wait(bit, Duration::from_millis(500).to_ticks());
    let elapsed = System::get_tick_count() - start;
    StatusSignal::clear(bit);
    test_assert!(bits & bit != 0);
    test_assert!(elapsed < Duration::from_millis(50).to_ticks(), "wait on a set bit blocked {elapsed} ticks");

    let start = System::get_tick_count();
    let bits = StatusSignal::wait(bit, Duration::from_millis(100).to_ticks());
    let elapsed = System::get_tick_count() - start;
    test_assert!(bits & bit == 0);
    test_assert!(elapsed >= Duration::from_millis(100).to_ticks(), "wait timed out after {elapsed} ticks");
    Ok(())
}

fn test_set_app_error() -> Result<()> {
    let bit = EventBits::from(ErrorFlag::DisplayHeader);
    let before = ErrorSignal::get();

    let ok: Result<()> = Ok(());
    set_app_error!(ok, ErrorFlag::DisplayHeader);
    test_assert_eq!(ErrorSignal::get(), before);

    let err: Result<()> = Err(Error::Unhandled("test error"));
    set_app_error!(err, ErrorFlag::DisplayHeader);
    let after = ErrorSignal::get();
    ErrorSignal::clear(bit);

    test_assert_eq!(after, before | bit);
    test_assert_eq!(ErrorSignal::get(), before & !bit);
    Ok(())
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_status_flags,
        test_status_check_signal,
        test_error_flags,
        test_display_flags,
        test_status_signal_set_clear,
        test_status_signal_wait,
        test_set_app_error,
    );
}
