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

//! Repeating hardware timer tests: the callback fires at the requested
//! period, gets back the very data it was registered with, and stops on
//! cancel. Both a short period and the 60s one the firmware loops on.

use core::ffi::c_void;
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;

use osal_rs::log_debug;
use osal_rs::os::ToTick;
use osal_rs::os::types::TickType;
use osal_rs::os::{System, SystemFn};
use osal_rs::utils::Result;

use super::Timer;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};

const TAG: &str = "TimerTests";

const PERIOD_MS: i32 = 20;

/// What the timer is registered with: the callback must find it unchanged
static USER_DATA: u32 = 0xC0FF_EE00;

static TICKS: AtomicU32 = AtomicU32::new(0);
static BAD_USER_DATA: AtomicU32 = AtomicU32::new(0);

const MINUTE_PERIOD_MS: i32 = 60_000;

/// How far off the 60s period a tick may land: the alarm itself is exact,
/// the slack is the polling below and the host scheduler of the emulator
const MINUTE_TOLERANCE_MS: u64 = 500;

/// How often the test task looks at the counter while waiting a tick
const POLL_MS: u64 = 50;

static MINUTE_USER_DATA: u32 = 0x6000_0060;

static MINUTE_TICKS: AtomicU32 = AtomicU32::new(0);
static MINUTE_BAD_USER_DATA: AtomicU32 = AtomicU32::new(0);

extern "C" fn on_tick(user_data: *mut c_void) {
    let expected = &raw const USER_DATA;
    if user_data as *const u32 == expected && unsafe { *expected } == 0xC0FF_EE00 {
        TICKS.fetch_add(1, Ordering::Relaxed);
    } else {
        BAD_USER_DATA.fetch_add(1, Ordering::Relaxed);
    }
}

// Runs in the alarm IRQ on the board: no RTOS call, only the counter
extern "C" fn on_minute_tick(user_data: *mut c_void) {
    let expected = &raw const MINUTE_USER_DATA;
    if user_data as *const u32 == expected && unsafe { *expected } == 0x6000_0060 {
        MINUTE_TICKS.fetch_add(1, Ordering::Relaxed);
    } else {
        MINUTE_BAD_USER_DATA.fetch_add(1, Ordering::Relaxed);
    }
}

/// Polls `MINUTE_TICKS` until it goes past `seen` or `timeout_ms` runs out,
/// returns the RTOS tick count at which the change was noticed
fn wait_minute_tick(seen: u32, timeout_ms: u64) -> Option<TickType> {
    let start = System::get_tick_count();
    let timeout = Duration::from_millis(timeout_ms).to_ticks();
    loop {
        if MINUTE_TICKS.load(Ordering::Relaxed) != seen {
            return Some(System::get_tick_count());
        }
        if System::get_tick_count().wrapping_sub(start) > timeout {
            return None;
        }
        System::delay_with_to_tick(Duration::from_millis(POLL_MS));
    }
}

fn test_repeating_timer() -> Result<()> {
    TICKS.store(0, Ordering::Relaxed);
    BAD_USER_DATA.store(0, Ordering::Relaxed);

    let timer = Timer::add_repeating_ms(PERIOD_MS, &USER_DATA, on_tick)?;
    System::delay_with_to_tick(Duration::from_millis(10 * PERIOD_MS as u64 + PERIOD_MS as u64 / 2));
    timer.cancel();

    let ticks = TICKS.load(Ordering::Relaxed);
    log_debug!(TAG, "{ticks} ticks in {}ms", 10 * PERIOD_MS + PERIOD_MS / 2);
    test_assert_eq!(BAD_USER_DATA.load(Ordering::Relaxed), 0, "callback got a wrong user_data pointer");
    test_assert!((8..=11).contains(&ticks), "{ticks} ticks, 10 expected");

    // Cancelled: nothing fires any more
    System::delay_with_to_tick(Duration::from_millis(5 * PERIOD_MS as u64));
    test_assert_eq!(TICKS.load(Ordering::Relaxed), ticks, "timer still running after cancel");
    Ok(())
}

/// The 60s loop: nothing before the minute, one callback per minute with the
/// registered data, and silence for a whole period after cancel. Takes ~3min
fn test_repeating_timer_60s() -> Result<()> {
    MINUTE_TICKS.store(0, Ordering::Relaxed);
    MINUTE_BAD_USER_DATA.store(0, Ordering::Relaxed);

    let period = Duration::from_millis(MINUTE_PERIOD_MS as u64).to_ticks();
    let tolerance = Duration::from_millis(MINUTE_TOLERANCE_MS).to_ticks();
    let period_window = MINUTE_PERIOD_MS as u64 + MINUTE_TOLERANCE_MS;

    let start = System::get_tick_count();
    let timer = Timer::add_repeating_ms(MINUTE_PERIOD_MS, &MINUTE_USER_DATA, on_minute_tick)?;

    let first = wait_minute_tick(0, period_window);
    let second = first.and_then(|_| wait_minute_tick(1, period_window));
    timer.cancel();

    let first = first.map(|at| at.wrapping_sub(start));
    let second = second.zip(first).map(|(at, first)| at.wrapping_sub(start) - first);
    log_debug!(TAG, "60s timer: first tick after {first:?} ticks, second {second:?} ticks later");

    test_assert_eq!(MINUTE_BAD_USER_DATA.load(Ordering::Relaxed), 0, "callback got a wrong user_data pointer");
    test_assert!(first.is_some(), "no callback within {period_window}ms");
    let first = first.unwrap_or_default();
    test_assert!(first + tolerance >= period, "first callback after {first} ticks, before the minute");
    test_assert!(first <= period + tolerance, "first callback after {first} ticks, {period} expected");
    test_assert!(second.is_some(), "timer did not repeat within {period_window}ms");
    let second = second.unwrap_or_default();
    test_assert!(second.abs_diff(period) <= tolerance, "second callback {second} ticks after the first, {period} expected");

    // Cancelled: a whole period goes by without callbacks
    let ticks = MINUTE_TICKS.load(Ordering::Relaxed);
    test_assert_eq!(ticks, 2, "{ticks} callbacks in two minutes");
    test_assert!(wait_minute_tick(ticks, period_window).is_none(), "timer still running after cancel");
    Ok(())
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_repeating_timer,
        //test_repeating_timer_60s,
    );
}
