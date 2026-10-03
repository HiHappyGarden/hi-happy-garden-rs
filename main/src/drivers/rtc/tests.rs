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

//! RTC tests on the real clocks: the DS3231 on I2C0 (battery backed) and the
//! RP2350 POWMAN timer the application reads at runtime.
//!
//! The only write puts back the time just read, so the board clock is left
//! as it was (give or take the sub-second the DS3231 cannot store).

use core::time::Duration;

use osal_rs::log_debug;
use osal_rs::os::{MutexFn, System};
use osal_rs::utils::{Error, Result};

use super::RTC;
use crate::drivers::date_time::DateTime;
use crate::drivers::i2c::I2C;
use crate::drivers::platform::{I2C0_INSTANCE, I2C_BAUDRATE};
use crate::tests::{TestStats, hardware, run_tests, test_assert, test_assert_eq};
use crate::traits::hardware::HardwareFn;
use crate::traits::rtc::RTC as RTCFn;
use crate::traits::state::Initializable;

const TAG: &str = "RtcTests";

/// A second driver instance bound to the DS3231, to compare it with POWMAN.
fn ds3231() -> Result<RTC> {
    let mut i2c = I2C::<{I2C0_INSTANCE}, {I2C_BAUDRATE}>::new();
    i2c.init()?;
    let mut rtc = RTC::shared();
    rtc.set_i2c(i2c);
    rtc.init()?;
    Ok(rtc)
}

fn test_without_i2c() -> Result<()> {
    let mut rtc = RTC::shared();
    test_assert!(matches!(rtc.init(), Err(Error::NullPtr)));
    test_assert!(matches!(RTCFn::get_timestamp(&rtc), Err(Error::NullPtr)));
    test_assert!(matches!(rtc.get_rtc_timestamp(), Err(Error::NullPtr)));
    test_assert!(matches!(RTCFn::set_timestamp(&rtc, 0), Err(Error::NullPtr)));
    test_assert!(rtc.is_to_synch());
    Ok(())
}

fn test_time_is_set() -> Result<()> {
    let ts = hardware().get_rtc().lock()?.get_timestamp()?;
    log_debug!(TAG, "POWMAN: {}", DateTime::from_timestamp(ts)?);
    test_assert!(ts > RTC::MINIMUM_DATE, "board time not set: {ts}");

    let dt = hardware().get_rtc().lock()?.timestamp_to_datetime(false)?;
    test_assert!(dt.is_valid());
    Ok(())
}

fn test_ds3231_matches_powman() -> Result<()> {
    let rtc = ds3231()?;
    let ds = rtc.get_rtc_timestamp()?;
    let powman = hardware().get_rtc().lock()?.get_timestamp()?;
    log_debug!(TAG, "DS3231: {} POWMAN: {}", DateTime::from_timestamp(ds)?, DateTime::from_timestamp(powman)?);
    test_assert!(ds > RTC::MINIMUM_DATE, "DS3231 not set or battery flat: {ds}");
    test_assert!((ds - powman).abs() <= 2, "clocks drifted: DS3231 {ds} POWMAN {powman}");
    test_assert!(rtc.is_to_synch());
    Ok(())
}

fn test_clock_runs() -> Result<()> {
    let rtc = hardware().get_rtc();
    let t0 = rtc.lock()?.get_timestamp()?;
    System::delay_with_to_tick(Duration::from_millis(2_100));
    let t1 = rtc.lock()?.get_timestamp()?;
    test_assert!((2..=3).contains(&(t1 - t0)), "POWMAN advanced {}s in 2.1s", t1 - t0);

    let ds = ds3231()?;
    let d0 = ds.get_rtc_timestamp()?;
    System::delay_with_to_tick(Duration::from_millis(2_100));
    let d1 = ds.get_rtc_timestamp()?;
    test_assert!((2..=3).contains(&(d1 - d0)), "DS3231 advanced {}s in 2.1s", d1 - d0);
    Ok(())
}

fn test_set_roundtrip() -> Result<()> {
    let rtc = hardware().get_rtc();
    let now = rtc.lock()?.get_timestamp()?;
    test_assert!(now > RTC::MINIMUM_DATE, "board time not set, refusing to write it");

    rtc.lock()?.set_timestamp(now)?;

    let powman = rtc.lock()?.get_timestamp()?;
    let ds = ds3231()?.get_rtc_timestamp()?;
    test_assert!((now..=now + 1).contains(&powman), "POWMAN {powman} after writing {now}");
    test_assert!((now..=now + 1).contains(&ds), "DS3231 {ds} after writing {now}");
    Ok(())
}

fn test_ds3231_calendar_fields() -> Result<()> {
    // The DS3231 stores BCD fields: a full read gives back a coherent date
    let ds = ds3231()?.get_rtc_timestamp()?;
    let dt = DateTime::from_timestamp(ds)?;
    test_assert!((1..=12).contains(&dt.month));
    test_assert!(dt.mday >= 1 && dt.mday <= DateTime::days_in_month(dt.month, dt.year));
    test_assert!(dt.year >= 2020 && dt.year < 2100);
    test_assert_eq!(DateTime::from_timestamp(dt.to_timestamp())?, dt);
    Ok(())
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_without_i2c,
        test_time_is_set,
        test_ds3231_matches_powman,
        test_ds3231_calendar_fields,
        test_clock_runs,
        test_set_roundtrip,
    );
}
