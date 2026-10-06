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
//! cancel.

use core::ffi::c_void;
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;

use osal_rs::log_debug;
use osal_rs::os::System;
use osal_rs::utils::Result;

use super::Timer;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};

const TAG: &str = "TimerTests";

const PERIOD_MS: i32 = 20;

/// What the timer is registered with: the callback must find it unchanged
static USER_DATA: u32 = 0xC0FF_EE00;

static TICKS: AtomicU32 = AtomicU32::new(0);
static BAD_USER_DATA: AtomicU32 = AtomicU32::new(0);

extern "C" fn on_tick(user_data: *mut c_void) {
    let expected = &raw const USER_DATA;
    if user_data as *const u32 == expected && unsafe { *expected } == 0xC0FF_EE00 {
        TICKS.fetch_add(1, Ordering::Relaxed);
    } else {
        BAD_USER_DATA.fetch_add(1, Ordering::Relaxed);
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

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_repeating_timer,
    );
}
