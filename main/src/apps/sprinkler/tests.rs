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
//!
//! The disbursement thread is already running (started by
//! `test_helpers::setup`): it is driven through `QUEUE_ZONES` and observed
//! through the `Disbursement*` status flags, since outputs cannot be read
//! back from the GPIO driver.

use core::time::Duration;

use osal_rs::os::types::EventBits;
use osal_rs::os::{BytesHasLen, QueueStreamed, QueueStreamedFn, System, ToTick};
use osal_rs::utils::Result;

use super::commons::Status;
use super::schedule::{Schedule, ScheduleController, ScheduleZone};
use super::zone::{Zone, ZoneController, ZoneRelay};
use super::{QUEUE_ZONES, Sprinkler, SprinklerTriggerData, DISBURSEMENT_MASK};
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::drivers::date_time::DateTime;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::signal::Signal;

const TAG: &str = "SprinklerTests";

/// A valid local time, with the weekday computed from the date.
fn at(year: i32, month: u8, mday: u8, hour: u8, minute: u8) -> Result<DateTime> {
    DateTime::from_timestamp(DateTime::new(year, month, 0, mday, hour, minute, 0)?.to_timestamp())
}

fn disbursement_in_progress() -> bool {
    StatusSignal::get() & DISBURSEMENT_MASK != 0
}

fn clear_disbursement() {
    StatusSignal::clear(DISBURSEMENT_MASK);
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
    schedule.zones[0] = ScheduleZone { relay: ZoneRelay::Relay0, minutes: 10 };
    schedule
}

/// Polls `cond` every 50 ms for at most `timeout`.
fn wait_until(timeout: Duration, cond: impl Fn() -> bool) -> bool {
    const STEP: Duration = Duration::from_millis(50);
    let mut waited = Duration::ZERO;
    while !cond() {
        if waited >= timeout {
            return false;
        }
        System::delay_with_to_tick(STEP);
        waited += STEP;
    }
    true
}

fn relay_flag(relay: ZoneRelay) -> EventBits {
    (match relay {
        ZoneRelay::Relay0 => StatusFlag::DisbursementZoneRelay0,
        ZoneRelay::Relay1 => StatusFlag::DisbursementZoneRelay1,
        ZoneRelay::Relay2 => StatusFlag::DisbursementZoneRelay2,
        ZoneRelay::Relay3 => StatusFlag::DisbursementZoneRelay3,
    }) as EventBits
}

/// Runs `test` on the shared schedules and zones, then restores them and
/// clears the disbursement flags.
fn with_shared_restored(test: impl FnOnce() -> Result<()>) -> Result<()> {
    let schedules = *ScheduleController::shared();
    let zones = *ZoneController::shared();
    clear_disbursement();
    let ret = test();
    *ScheduleController::shared() = schedules;
    *ZoneController::shared() = zones;
    clear_disbursement();
    ret
}

fn test_status_conversion() -> Result<()> {
    for status in [Status::UNACTIVE, Status::ACTIVE, Status::RUN] {
        test_assert_eq!(Status::from(u8::from(status)), status);
    }
    test_assert_eq!(Status::from(3), Status::UNACTIVE);
    test_assert_eq!(Status::from(255), Status::UNACTIVE);
    Ok(())
}

fn test_status_from_str() -> Result<()> {
    // The display shows `as_str`, the zone detail screen parses it back
    for status in [Status::UNACTIVE, Status::ACTIVE, Status::RUN] {
        test_assert_eq!(Status::from(status.as_str()), status);
    }
    for text in ["", "active", "ACTIVE", "Active ", "Running"] {
        test_assert_eq!(Status::from(text), Status::UNACTIVE, "{text:?}");
    }
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
    schedule.months = 0x0001; // January
    test_assert!(schedule.executable(&at(2024, 1, 15, 10, 0)?), "January schedule skipped in January");
    test_assert!(!schedule.executable(&at(2024, 2, 15, 10, 0)?), "January schedule ran in February");

    schedule.months = 0x0800; // December
    test_assert!(schedule.executable(&at(2024, 12, 1, 10, 0)?), "December schedule skipped in December");

    schedule.months = 0x0020 | 0x0040 | 0x0080; // June..August
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
    with_shared_restored(|| {
        // Zone 0 ACTIVE so that the thread really waters (1 minute)
        ZoneController::shared()[0].status = Status::ACTIVE;
        for (idx, schedule) in ScheduleController::shared().into_iter().enumerate() {
            *schedule = active();
            schedule.zones[0].minutes = 1;
            // Only index 2 matches 12:00, index 3 would match too but comes later
            schedule.hour = match idx { 2 | 3 => 13, _ => 1 };
        }

        let mut sprinkler = Sprinkler::new();
        sprinkler.check(at(2024, 6, 17, 12, 0)?);
        test_assert_eq!(statuses(), [Status::ACTIVE, Status::ACTIVE, Status::RUN, Status::ACTIVE]);

        // The flag is raised by the thread, asynchronously
        test_assert!(wait_until(Duration::from_millis(1_500), disbursement_in_progress),
            "disbursement never started");

        // One disbursement at a time: nothing else starts meanwhile
        sprinkler.check(at(2024, 6, 17, 12, 0)?);
        test_assert_eq!(statuses(), [Status::ACTIVE, Status::ACTIVE, Status::RUN, Status::ACTIVE]);
        Ok(())
    })
}

fn test_check_idle() -> Result<()> {
    let backup = *ScheduleController::shared();
    clear_disbursement();

    let ret = (|| {
        for schedule in ScheduleController::shared().into_iter() {
            *schedule = active();
            schedule.hour = 1;
        }
        Sprinkler::new().check(at(2024, 6, 17, 12, 0)?);
        test_assert!(!disbursement_in_progress());
        test_assert!(ScheduleController::shared().into_iter().all(|s| s.status == Status::ACTIVE));
        Ok(())
    })();

    *ScheduleController::shared() = backup;
    clear_disbursement();
    ret
}

fn test_trigger_data_default() -> Result<()> {
    let data = SprinklerTriggerData::default();
    test_assert_eq!(data.zone, Zone::new(ZoneRelay::Relay0));
    test_assert_eq!(data.duration, 0);
    test_assert_eq!(data.weight, 0);
    test_assert_eq!(data.status, Status::UNACTIVE);
    Ok(())
}

fn test_trigger_data_queue_roundtrip() -> Result<()> {
    // A worst-case item (full description) must fit the queue message
    // size and come back unchanged
    let description = "x".repeat(DISPLAY_INPUT_MAX_SIZE + 4);
    let mut zone = Zone::new_with_description(&description, ZoneRelay::Relay2);
    zone.weight = 7;
    zone.status = Status::ACTIVE;
    let sent = SprinklerTriggerData {
        zone,
        start: at(2024, 6, 17, 8, 30)?,
        duration: 45 * 60,
        weight: 7,
        status: Status::RUN,
    };
    // Exact match: catches a field added without updating SERIALIZED_SIZE
    let mut wire = alloc::vec::Vec::new();
    test_assert_eq!(osal_rs_serde::to_dyn_bytes(&sent, &mut wire)
        .map_err(|_| osal_rs::utils::Error::Unhandled("serialization failed"))?, SprinklerTriggerData::SERIALIZED_SIZE);
    test_assert_eq!(sent.len(), SprinklerTriggerData::SERIALIZED_SIZE);

    let queue = QueueStreamed::<SprinklerTriggerData>::new(2, SprinklerTriggerData::SERIALIZED_SIZE as osal_rs::os::types::UBaseType)?;
    queue.post(&sent, Duration::from_millis(100).to_ticks())?;
    let mut received = SprinklerTriggerData::default();
    queue.fetch(&mut received, Duration::from_millis(100).to_ticks())?;

    test_assert_eq!(received.zone, sent.zone);
    test_assert!(received.start == sent.start, "start changed on the wire");
    test_assert_eq!(received.duration, sent.duration);
    test_assert_eq!(received.weight, sent.weight);
    test_assert_eq!(received.status, sent.status);
    Ok(())
}

fn test_schedule_zones_iteration() -> Result<()> {
    // `handle_schedule_not_running` walks `&Schedule`: it must
    // yield each zone once, in order, and then stop
    let mut schedule = Schedule::new();
    for (idx, zone) in schedule.zones.iter_mut().enumerate() {
        zone.minutes = idx as u8 + 1;
    }
    let expected = schedule.zones;
    let mut yielded = [ScheduleZone::new(ZoneRelay::Relay0); ZoneController::SIZE + 1];
    let mut count = 0;
    for (slot, zone) in yielded.iter_mut().zip((&schedule).into_iter().take(ZoneController::SIZE + 1)) {
        *slot = zone;
        count += 1;
    }
    test_assert_eq!(count, ZoneController::SIZE, "iterator does not stop after the last zone");
    test_assert_eq!(&yielded[..ZoneController::SIZE], &expected[..]);
    Ok(())
}

fn test_handle_marks_running() -> Result<()> {
    with_shared_restored(|| {
        // Zones left UNACTIVE: nothing to water, the schedule still runs
        for zone in ZoneController::shared().iter_mut() {
            zone.status = Status::UNACTIVE;
        }
        let mut schedule = active();
        test_assert!(Sprinkler::handle_schedule_not_running(&mut schedule, at(2024, 6, 17, 12, 0)?));
        test_assert_eq!(schedule.status, Status::RUN);
        Ok(())
    })
}

fn test_handle_relay_mismatch() -> Result<()> {
    with_shared_restored(|| {
        // Schedule zones out of the zone controller order: refused untouched
        let mut schedule = active();
        schedule.zones.swap(0, 1);
        test_assert!(!Sprinkler::handle_schedule_not_running(&mut schedule, at(2024, 6, 17, 12, 0)?));
        test_assert_eq!(schedule.status, Status::ACTIVE);
        Ok(())
    })
}

fn test_disbursement_cycle() -> Result<()> {
    // Thread side: an ACTIVE item opens its relay, counts down its
    // duration (seconds) and then closes it again
    with_shared_restored(|| {
        let flag = relay_flag(ZoneRelay::Relay3);
        let mut zone = Zone::new(ZoneRelay::Relay3);
        zone.status = Status::ACTIVE;
        let data = SprinklerTriggerData {
            zone,
            start: at(2024, 6, 17, 12, 0)?,
            duration: 2,
            weight: 0,
            status: Status::ACTIVE,
        };
        QUEUE_ZONES.get().ok_or(osal_rs::utils::Error::Unhandled("queue not initialized"))?
            .post(&data, Duration::from_millis(100).to_ticks())?;

        test_assert!(wait_until(Duration::from_millis(1_500), || StatusSignal::get() & flag != 0),
            "relay 3 disbursement never started");
        test_assert!(StatusSignal::get() & DISBURSEMENT_MASK == flag, "other relays started");
        test_assert!(wait_until(Duration::from_millis(6_000), || StatusSignal::get() & flag == 0),
            "relay 3 disbursement never stopped");
        Ok(())
    })
}

fn test_check_waters_active_zone() -> Result<()> {
    // End to end: a matching schedule on an ACTIVE zone reaches the thread,
    // which opens that zone's relay. The item stays queued in the thread for
    // its whole minute: keep this test last.
    with_shared_restored(|| {
        for zone in ZoneController::shared().iter_mut() {
            zone.status = Status::UNACTIVE;
        }
        ZoneController::shared()[1].status = Status::ACTIVE;
        for schedule in ScheduleController::shared().into_iter() {
            *schedule = Schedule::new();
        }
        let schedule = ScheduleController::shared().into_iter().next()
            .ok_or(osal_rs::utils::Error::Unhandled("no schedules"))?;
        schedule.status = Status::ACTIVE;
        schedule.zones[0].minutes = 1; // zone 0 is UNACTIVE: skipped
        schedule.zones[1].minutes = 1;

        Sprinkler::new().check(at(2024, 6, 17, 12, 0)?);

        let flag = relay_flag(ZoneRelay::Relay1);
        test_assert!(wait_until(Duration::from_millis(1_500), || StatusSignal::get() & flag != 0),
            "scheduled zone never started");
        test_assert!(StatusSignal::get() & DISBURSEMENT_MASK == flag, "an unactive zone started");
        Ok(())
    })
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_status_conversion,
        test_status_from_str,
        test_every_minute,
        test_time_of_day,
        test_days_of_week,
        test_months,
        test_running_not_executable,
        test_inactive_not_executable,
        test_check_idle,
        test_check_starts_matching_schedule,
        test_trigger_data_default,
        test_trigger_data_queue_roundtrip,
        test_schedule_zones_iteration,
        test_handle_marks_running,
        test_handle_relay_mismatch,
        test_disbursement_cycle,
        // Last: leaves a one-minute disbursement running in the thread
        test_check_waters_active_zone,
    );

    super::zone::tests::run_all_tests(stats);
    super::schedule::tests::run_all_tests(stats);
}
