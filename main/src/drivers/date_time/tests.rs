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

//! `DateTime` tests: pure calendar logic, no peripheral involved.
//!
//! Timezone and DST live in module statics shared with the whole firmware,
//! so every test that changes them restores the neutral state (UTC, DST off)
//! before returning, pass or fail.

use alloc::format;

use osal_rs::log_debug;
use osal_rs::utils::Result;

use super::DateTime;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};

const TAG: &str = "DateTimeTests";

/// EU rule: last Sunday of March 02:00 -> last Sunday of October 03:00
const LAST_SUNDAY: u8 = 0xFF;

fn reset_locale() {
    DateTime::set_timezone(0);
    DateTime::set_daylight_saving_time_whit_param(false, 0, 0, 0, 0, 0, 0);
}

fn set_eu_locale() {
    DateTime::set_timezone(60);
    DateTime::set_daylight_saving_time_whit_param(true, 3, LAST_SUNDAY, 2, 10, LAST_SUNDAY, 3);
}

/// Runs `test` with the locale reset afterwards, whatever the outcome.
fn with_locale_reset(test: impl FnOnce() -> Result<()>) -> Result<()> {
    let ret = test();
    reset_locale();
    ret
}

fn test_days_in_month() -> Result<()> {
    test_assert_eq!(DateTime::days_in_month(1, 2023), 31);
    test_assert_eq!(DateTime::days_in_month(4, 2023), 30);
    test_assert_eq!(DateTime::days_in_month(2, 2023), 28);
    test_assert_eq!(DateTime::days_in_month(2, 2024), 29);
    // Century rule: 1900 and 2100 are not leap years, 2000 is
    test_assert_eq!(DateTime::days_in_month(2, 1900), 28);
    test_assert_eq!(DateTime::days_in_month(2, 2000), 29);
    test_assert_eq!(DateTime::days_in_month(2, 2100), 28);
    test_assert_eq!(DateTime::days_in_month(0, 2024), 0);
    test_assert_eq!(DateTime::days_in_month(13, 2024), 0);
    Ok(())
}

fn test_new_validation() -> Result<()> {
    test_assert!(DateTime::new(2024, 2, 4, 29, 23, 59, 59).is_ok());
    test_assert!(DateTime::new(2023, 2, 3, 29, 0, 0, 0).is_err(), "29 Feb on a non leap year accepted");
    test_assert!(DateTime::new(2024, 0, 0, 1, 0, 0, 0).is_err());
    test_assert!(DateTime::new(2024, 13, 0, 1, 0, 0, 0).is_err());
    test_assert!(DateTime::new(2024, 1, 0, 0, 0, 0, 0).is_err());
    test_assert!(DateTime::new(2024, 4, 0, 31, 0, 0, 0).is_err());
    test_assert!(DateTime::new(2024, 1, 0, 1, 24, 0, 0).is_err());
    test_assert!(DateTime::new(2024, 1, 0, 1, 0, 60, 0).is_err());
    test_assert!(DateTime::new(2024, 1, 0, 1, 0, 0, 60).is_err());

    test_assert!(DateTime::new_date(2024, 2, 29).is_ok());
    test_assert!(DateTime::new_date(2023, 2, 29).is_err());
    test_assert!(DateTime::new_date(2024, 13, 1).is_err());

    test_assert!(DateTime::new_time(23, 59, 59).is_ok());
    test_assert!(DateTime::new_time(24, 0, 0).is_err());
    test_assert!(DateTime::new_time(0, 60, 0).is_err());
    test_assert!(DateTime::new_time(0, 0, 60).is_err());
    Ok(())
}

fn test_epoch() -> Result<()> {
    let dt = DateTime::from_timestamp(0)?;
    log_debug!(TAG, "epoch: {:?}", dt);
    test_assert_eq!(dt, DateTime::default());
    test_assert_eq!(dt.wday, 4); // Thursday
    test_assert_eq!(dt.to_timestamp(), 0);
    Ok(())
}

fn test_known_dates() -> Result<()> {
    // (timestamp, year, month, mday, wday, hour, minute, second)
    let cases: [(i64, i32, u8, u8, u8, u8, u8, u8); 6] = [
        (951_782_400, 2000, 2, 29, 2, 0, 0, 0),         // leap day of a 400-year
        (1_577_836_800, 2020, 1, 1, 3, 0, 0, 0),        // RTC::MINIMUM_DATE
        (1_709_251_199, 2024, 2, 29, 4, 23, 59, 59),    // last second of a leap day
        (1_735_689_599, 2024, 12, 31, 2, 23, 59, 59),   // last second of a leap year
        (2_147_483_647, 2038, 1, 19, 2, 3, 14, 7),      // i32 overflow (Y2038)
        (4_102_444_800, 2100, 1, 1, 5, 0, 0, 0),        // non leap century
    ];

    for (ts, year, month, mday, wday, hour, minute, second) in cases {
        let dt = DateTime::from_timestamp(ts)?;
        log_debug!(TAG, "{ts} -> {dt}");
        test_assert_eq!((dt.year, dt.month, dt.mday, dt.wday, dt.hour, dt.minute, dt.second),
                        (year, month, mday, wday, hour, minute, second));
        test_assert_eq!(dt.to_timestamp(), ts);
    }
    Ok(())
}

fn test_negative_timestamp() -> Result<()> {
    let dt = DateTime::from_timestamp(-1)?;
    log_debug!(TAG, "-1 -> {:?}", dt);
    test_assert_eq!((dt.year, dt.month, dt.mday, dt.hour, dt.minute, dt.second), (1969, 12, 31, 23, 59, 59));
    test_assert_eq!(dt.to_timestamp(), -1);
    // 1969-12-31 was a Wednesday
    test_assert_eq!(dt.wday, 3);
    Ok(())
}

fn test_roundtrip() -> Result<()> {
    // One sample every ~97 days over 1970..2106 hits every month/weekday combination
    let mut ts: i64 = 0;
    while ts < 4_300_000_000 {
        let dt = DateTime::from_timestamp(ts)?;
        test_assert_eq!(dt.to_timestamp(), ts);
        test_assert!(dt.wday < 7);
        ts += 97 * DateTime::SECONDS_PER_DAY + 3_661;
    }
    Ok(())
}

fn test_ordering() -> Result<()> {
    let a = DateTime::from_timestamp(1_700_000_000)?;
    let b = DateTime::from_timestamp(1_700_000_001)?;
    let c = DateTime::from_timestamp(1_800_000_000)?;
    test_assert!(a < b);
    test_assert!(b < c);
    test_assert_eq!(a, DateTime::from_timestamp(1_700_000_000)?);
    Ok(())
}

fn test_is_valid() -> Result<()> {
    test_assert!(!DateTime::default().is_valid());
    test_assert!(DateTime::from_timestamp(1_577_836_800)?.is_valid());
    Ok(())
}

fn test_display() -> Result<()> {
    with_locale_reset(|| {
        let dt = DateTime::from_timestamp(1_709_251_199)?;
        test_assert_eq!(format!("{dt}"), "2024-02-29 23:59:59 UTC");

        DateTime::set_timezone(90);
        let dt = DateTime::from_timestamp_locale(0, true)?;
        test_assert_eq!(format!("{dt}"), "1970-01-01 01:30:00 (UTC+01:30)");
        Ok(())
    })
}

fn test_timezone() -> Result<()> {
    with_locale_reset(|| {
        let ts = 1_718_712_000; // 2024-06-18 12:00:00 UTC

        DateTime::set_timezone(120);
        let dt = DateTime::from_timestamp_locale(ts, true)?;
        test_assert!(dt.is_apply_timezone());
        test_assert!(!dt.is_apply_daylight_saving_time());
        test_assert_eq!((dt.hour, dt.minute), (14, 0));
        test_assert_eq!(dt.to_timestamp(), ts);

        DateTime::set_timezone(-300);
        let dt = DateTime::from_timestamp_locale(ts, true)?;
        test_assert_eq!((dt.mday, dt.hour), (18, 7));
        test_assert_eq!(dt.to_timestamp(), ts);

        // Crossing midnight backwards changes the day
        let dt = DateTime::from_timestamp_locale(1_718_668_800, true)?; // 2024-06-18 00:00 UTC
        test_assert_eq!((dt.mday, dt.hour), (17, 19));

        // locale = false ignores the configured offset
        let dt = DateTime::from_timestamp_locale(ts, false)?;
        test_assert!(!dt.is_apply_timezone());
        test_assert_eq!(dt.hour, 12);
        Ok(())
    })
}

fn test_last_sunday_of_month() -> Result<()> {
    test_assert_eq!(DateTime::find_last_weekday_of_month(2024, 3, 0), 31);
    test_assert_eq!(DateTime::find_last_weekday_of_month(2024, 10, 0), 27);
    test_assert_eq!(DateTime::find_last_weekday_of_month(2025, 3, 0), 30);
    test_assert_eq!(DateTime::find_last_weekday_of_month(2025, 10, 0), 26);
    test_assert_eq!(DateTime::find_last_weekday_of_month(2026, 3, 0), 29);
    test_assert_eq!(DateTime::find_last_weekday_of_month(2026, 10, 0), 25);
    Ok(())
}

fn test_dst_eu_season() -> Result<()> {
    with_locale_reset(|| {
        set_eu_locale();

        let winter = DateTime::from_timestamp_locale(1_705_320_000, true)?; // 2024-01-15 12:00 UTC
        test_assert!(!winter.is_apply_daylight_saving_time());
        test_assert_eq!(winter.hour, 13);
        test_assert_eq!(winter.to_timestamp(), 1_705_320_000);

        let summer = DateTime::from_timestamp_locale(1_719_835_200, true)?; // 2024-07-01 12:00 UTC
        test_assert!(summer.is_apply_daylight_saving_time());
        test_assert_eq!(summer.hour, 14);
        test_assert_eq!(summer.to_timestamp(), 1_719_835_200);

        let disabled_ts = 1_719_835_200;
        DateTime::set_daylight_saving_time(false);
        let dt = DateTime::from_timestamp_locale(disabled_ts, true)?;
        test_assert!(!dt.is_apply_daylight_saving_time());
        test_assert_eq!(dt.hour, 13);
        Ok(())
    })
}

fn test_dst_eu_spring_forward() -> Result<()> {
    with_locale_reset(|| {
        set_eu_locale();

        // EU: on 2024-03-31 at 01:00 UTC clocks jump from 02:00 CET to 03:00 CEST
        let before = DateTime::from_timestamp_locale(1_711_846_799, true)?; // 00:59:59 UTC
        test_assert!(!before.is_apply_daylight_saving_time(), "DST active before switch: {before}");
        test_assert_eq!((before.hour, before.minute), (1, 59));

        let after = DateTime::from_timestamp_locale(1_711_846_800, true)?; // 01:00:00 UTC
        test_assert!(after.is_apply_daylight_saving_time(), "DST not active after switch: {after}");
        test_assert_eq!((after.hour, after.minute), (3, 0));
        Ok(())
    })
}

fn test_dst_eu_fall_back() -> Result<()> {
    with_locale_reset(|| {
        set_eu_locale();

        // EU: on 2024-10-27 at 01:00 UTC clocks go back from 03:00 CEST to 02:00 CET
        let before = DateTime::from_timestamp_locale(1_729_990_799, true)?; // 00:59:59 UTC
        test_assert!(before.is_apply_daylight_saving_time(), "DST not active before switch: {before}");
        test_assert_eq!((before.hour, before.minute), (2, 59));

        let after = DateTime::from_timestamp_locale(1_729_992_600, true)?; // 01:30:00 UTC
        test_assert!(!after.is_apply_daylight_saving_time(), "DST still active after switch: {after}");
        test_assert_eq!((after.hour, after.minute), (2, 30));
        Ok(())
    })
}

fn test_dst_southern_hemisphere() -> Result<()> {
    with_locale_reset(|| {
        // DST from 1st October to 1st April
        DateTime::set_timezone(600);
        DateTime::set_daylight_saving_time_whit_param(true, 10, 1, 2, 4, 1, 3);

        let january = DateTime::from_timestamp_locale(1_705_320_000, true)?; // 2024-01-15
        test_assert!(january.is_apply_daylight_saving_time());

        let july = DateTime::from_timestamp_locale(1_719_835_200, true)?; // 2024-07-01
        test_assert!(!july.is_apply_daylight_saving_time());

        let november = DateTime::from_timestamp_locale(1_730_462_400, true)?; // 2024-11-01
        test_assert!(november.is_apply_daylight_saving_time());
        Ok(())
    })
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_days_in_month,
        test_new_validation,
        test_epoch,
        test_known_dates,
        test_negative_timestamp,
        test_roundtrip,
        test_ordering,
        test_is_valid,
        test_display,
        test_timezone,
        test_last_sunday_of_month,
        test_dst_eu_season,
        test_dst_eu_spring_forward,
        test_dst_eu_fall_back,
        test_dst_southern_hemisphere,
    );
}
