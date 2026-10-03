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


//! Sprinkler tests: the scheduling rule (`Schedule::executable`) and the
//! `Sprinkler::check` tick of the main FSM. The zone and schedule AT
//! handlers live in `zone::tests` and `schedule::tests`.
//!
//! `check` works on the shared schedules: they are restored, together with
//! the disbursement flag, at the end of each test.

use core::sync::atomic::Ordering;

use osal_rs::utils::Result;

use super::commons::Status;
use super::schedule::{Schedule, ScheduleController};
use super::zone::ZoneRelay;
use super::{DISBURSEMENT_IN_PROGRESS, Sprinkler};
use crate::drivers::date_time::DateTime;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};

const TAG: &str = "SprinklerTests";

/// A valid local time, with the weekday computed from the date.
fn at(year: i32, month: u8, mday: u8, hour: u8, minute: u8) -> Result<DateTime> {
    DateTime::from_timestamp(DateTime::new(year, month, 0, mday, hour, minute, 0)?.to_timestamp())
}

fn statuses() -> [Status; ScheduleController::SIZE] {
    let mut statuses = [Status::UNACTIVE; ScheduleController::SIZE];
    for (schedule, status) in ScheduleController::shared().into_iter().zip(statuses.iter_mut()) {
        *status = schedule.status;
    }
    statuses
}

fn active() -> Schedule {
    let mut schedule = Schedule::new();
    schedule.status = Status::ACTIVE;
    schedule.zones[0] = Some((ZoneRelay::Relay0, 10));
    schedule
}

fn test_status_conversion() -> Result<()> {
    for status in [Status::UNACTIVE, Status::ACTIVE, Status::RUN] {
        test_assert_eq!(Status::from(u8::from(status)), status);
    }
    test_assert_eq!(Status::from(3), Status::UNACTIVE);
    test_assert_eq!(Status::from(255), Status::UNACTIVE);
    Ok(())
}

fn test_every_minute() -> Result<()> {
    // All fields NOT_SET: runs at any time
    let schedule = active();
    test_assert!(schedule.executable(&at(2024, 6, 17, 0, 0)?));
    test_assert!(schedule.executable(&at(2024, 12, 31, 23, 59)?));
    Ok(())
}

fn test_time_of_day() -> Result<()> {
    // Stored 1-based: hour 9 / minute 31 mean 08:30
    let mut schedule = active();
    schedule.hour = 9;
    schedule.minute = 31;
    test_assert!(schedule.executable(&at(2024, 6, 17, 8, 30)?));
    test_assert!(!schedule.executable(&at(2024, 6, 17, 8, 31)?));
    test_assert!(!schedule.executable(&at(2024, 6, 17, 9, 30)?));

    schedule.hour = 1;
    schedule.minute = 1;
    test_assert!(schedule.executable(&at(2024, 6, 17, 0, 0)?), "midnight not matched");

    schedule.hour = 24;
    schedule.minute = 60;
    test_assert!(schedule.executable(&at(2024, 6, 17, 23, 59)?));

    // Hour only: every minute of that hour
    schedule.minute = Schedule::NOT_SET;
    schedule.hour = 7;
    test_assert!(schedule.executable(&at(2024, 6, 17, 6, 0)?));
    test_assert!(schedule.executable(&at(2024, 6, 17, 6, 59)?));
    test_assert!(!schedule.executable(&at(2024, 6, 17, 7, 0)?));
    Ok(())
}

fn test_days_of_week() -> Result<()> {
    let mut schedule = active();
    schedule.days = 0x02 | 0x08; // Monday | Wednesday
    test_assert!(schedule.executable(&at(2024, 6, 17, 10, 0)?), "Monday not matched");
    test_assert!(!schedule.executable(&at(2024, 6, 18, 10, 0)?), "Tuesday matched");
    test_assert!(schedule.executable(&at(2024, 6, 19, 10, 0)?), "Wednesday not matched");

    schedule.days = 0x01; // Sunday
    test_assert!(schedule.executable(&at(2024, 6, 16, 10, 0)?), "Sunday not matched");
    schedule.days = 0x40; // Saturday
    test_assert!(schedule.executable(&at(2024, 6, 15, 10, 0)?), "Saturday not matched");
    Ok(())
}

fn test_months() -> Result<()> {
    let mut schedule = active();
    schedule.month = 0x0001; // January
    test_assert!(schedule.executable(&at(2024, 1, 15, 10, 0)?), "January schedule skipped in January");
    test_assert!(!schedule.executable(&at(2024, 2, 15, 10, 0)?), "January schedule ran in February");

    schedule.month = 0x0800; // December
    test_assert!(schedule.executable(&at(2024, 12, 1, 10, 0)?), "December schedule skipped in December");

    schedule.month = 0x0020 | 0x0040 | 0x0080; // June..August
    test_assert!(schedule.executable(&at(2024, 7, 1, 10, 0)?), "summer schedule skipped in July");
    test_assert!(!schedule.executable(&at(2024, 9, 1, 10, 0)?), "summer schedule ran in September");
    Ok(())
}

fn test_running_not_executable() -> Result<()> {
    let mut schedule = active();
    schedule.status = Status::RUN;
    test_assert!(!schedule.executable(&at(2024, 6, 17, 10, 0)?));
    Ok(())
}

fn test_inactive_not_executable() -> Result<()> {
    // A disabled (or never configured) schedule must never water
    let mut schedule = active();
    schedule.status = Status::UNACTIVE;
    test_assert!(!schedule.executable(&at(2024, 6, 17, 10, 0)?), "UNACTIVE schedule is executable");
    test_assert!(!Schedule::new().executable(&at(2024, 6, 17, 10, 0)?), "factory schedule is executable");
    Ok(())
}

fn test_check_starts_matching_schedule() -> Result<()> {
    let controller = ScheduleController::shared();
    let backup = *controller;
    DISBURSEMENT_IN_PROGRESS.store(false, Ordering::Relaxed);

    let ret = (|| {
        for (idx, schedule) in ScheduleController::shared().into_iter().enumerate() {
            *schedule = active();
            // Only index 2 matches 12:00, index 3 would match too but comes later
            schedule.hour = match idx { 2 | 3 => 13, _ => 1 };
        }

        let mut sprinkler = Sprinkler::new();
        sprinkler.check(at(2024, 6, 17, 12, 0)?);

        test_assert_eq!(statuses(), [Status::ACTIVE, Status::ACTIVE, Status::RUN, Status::ACTIVE]);
        test_assert!(DISBURSEMENT_IN_PROGRESS.load(Ordering::Relaxed));

        // One disbursement at a time: nothing else starts meanwhile
        sprinkler.check(at(2024, 6, 17, 12, 0)?);
        test_assert_eq!(statuses(), [Status::ACTIVE, Status::ACTIVE, Status::RUN, Status::ACTIVE]);
        Ok(())
    })();

    *ScheduleController::shared() = backup;
    DISBURSEMENT_IN_PROGRESS.store(false, Ordering::Relaxed);
    ret
}

fn test_check_idle() -> Result<()> {
    let backup = *ScheduleController::shared();
    DISBURSEMENT_IN_PROGRESS.store(false, Ordering::Relaxed);

    let ret = (|| {
        for schedule in ScheduleController::shared().into_iter() {
            *schedule = active();
            schedule.hour = 1;
        }
        Sprinkler::new().check(at(2024, 6, 17, 12, 0)?);
        test_assert!(!DISBURSEMENT_IN_PROGRESS.load(Ordering::Relaxed));
        test_assert!(ScheduleController::shared().into_iter().all(|s| s.status == Status::ACTIVE));
        Ok(())
    })();

    *ScheduleController::shared() = backup;
    DISBURSEMENT_IN_PROGRESS.store(false, Ordering::Relaxed);
    ret
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_status_conversion,
        test_every_minute,
        test_time_of_day,
        test_days_of_week,
        test_months,
        test_running_not_executable,
        test_inactive_not_executable,
        test_check_idle,
        test_check_starts_matching_schedule,
    );

    super::zone::tests::run_all_tests(stats);
    super::schedule::tests::run_all_tests(stats);
}
