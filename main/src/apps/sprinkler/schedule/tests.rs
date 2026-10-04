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


//! Schedule tests: day/month bitmasks and the `AT+SCH` handler.
//!
//! Each AT test restores the shared schedules and the staging area. `sv`
//! is sent once, by `test_save_reload`, on the restored (unchanged)
//! schedules.

use alloc::format;
use alloc::string::String;

use at_parser_rs::Args;
use at_parser_rs::context::AtContext;
use osal_rs::utils::Result;

use super::{Day, MUTEX, Month, SCHEDULE_TMP, SHARED, Schedule, ScheduleController};
use crate::apps::parser::Parser;
use crate::apps::sprinkler::commons::Status;
use crate::apps::sprinkler::zone::{ZoneController, ZoneRelay};
use crate::apps::test_helpers::{at_body, at_error, logged};
use crate::apps::utils::deserialize_file;
use crate::drivers::platform::FS_CONFIG_DIR;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};

const TAG: &str = "ScheduleTests";

fn schedule(idx: usize) -> Schedule {
    unsafe { (*&raw const SHARED).schedules[idx] }
}

fn reset_staging() {
    unsafe { SCHEDULE_TMP = (0, Schedule::new()); }
}

/// Runs `test` on the shared schedules, then puts them and the staging area back.
fn with_schedules_restored(test: impl FnOnce(&'static mut ScheduleController) -> Result<()>) -> Result<()> {
    let backup = *ScheduleController::shared();
    reset_staging();
    let ret = test(ScheduleController::shared());
    *ScheduleController::shared() = backup;
    reset_staging();
    ret
}

fn set(controller: &mut ScheduleController, raw: &str) -> Result<String> {
    at_body(controller.set(ScheduleController::AT_RESP, Args { raw }))
}

fn test_day_map() -> Result<()> {
    let days = [Day::Sunday, Day::Monday, Day::Tuesday, Day::Wednesday, Day::Thursday, Day::Friday, Day::Saturday];
    for (wday, day) in days.into_iter().enumerate() {
        // Same numbering as DateTime::wday
        test_assert_eq!(u8::from(day), wday as u8);
        test_assert_eq!(day as u8, 1 << wday);
    }
    test_assert_eq!(Day::map(0), [None; 7]);
    test_assert_eq!(Day::map(0x7F), days.map(Some));
    test_assert_eq!(Day::map(0x02 | 0x40), [None, Some(Day::Monday), None, None, None, None, Some(Day::Saturday)]);
    Ok(())
}

fn test_month_map() -> Result<()> {
    let months = [
        Month::January, Month::February, Month::March, Month::April, Month::May, Month::June,
        Month::July, Month::August, Month::September, Month::October, Month::November, Month::December,
    ];
    for (idx, month) in months.into_iter().enumerate() {
        test_assert_eq!(month as u16, 1 << idx);
    }
    test_assert_eq!(Month::map(0), [None; 12]);
    test_assert_eq!(Month::map(0x0FFF), months.map(Some));
    let summer = Month::map(0x0020 | 0x0040 | 0x0080);
    test_assert_eq!(summer.iter().flatten().count(), 3);
    test_assert_eq!(summer[6], Some(Month::July));
    Ok(())
}

fn test_is_modified() -> Result<()> {
    test_assert!(!Schedule::is_modified(&Schedule::new()));
    let mut schedule = Schedule::new();
    schedule.minute = 1;
    test_assert!(Schedule::is_modified(&schedule));
    Ok(())
}

fn test_requires_login() -> Result<()> {
    with_schedules_restored(|controller| {
        test_assert_eq!(at_error(controller.set(ScheduleController::AT_RESP, Args { raw: "0,mi,1" }))?, Parser::NOT_LOGGED_RESPONSE);
        test_assert_eq!(at_error(controller.exec(ScheduleController::AT_RESP))?, Parser::NOT_LOGGED_RESPONSE);
        test_assert!(!Schedule::is_modified(unsafe { &(*&raw const SCHEDULE_TMP).1 }));
        Ok(())
    })
}

fn test_set_query_exec() -> Result<()> {
    with_schedules_restored(|controller| logged(|| {
        for raw in ["1,mi,31", "1,hr,9", "1,dy,10", "1,mo,224", "1,ds,Morning", "1,zn,1,10", "1,zn,2,5", "1,zn,1,15", "1,st,1"] {
            test_assert!(set(controller, raw)?.is_empty(), "{raw} rejected");
        }

        // query shows the staged schedule
        test_assert_eq!(
            at_body(controller.query(ScheduleController::AT_RESP))?,
            "1,31,9,10,224,Morning,[Relay 1=15, Relay 2=5, None, None]"
        );
        test_assert!(schedule(1) != unsafe { (*&raw const SCHEDULE_TMP).1 }, "set applied before exec");

        test_assert!(at_body(controller.exec(ScheduleController::AT_RESP))?.is_empty());
        let applied = schedule(1);
        test_assert_eq!((applied.minute, applied.hour, applied.days, applied.month), (31, 9, 10, 224));
        test_assert_eq!(applied.description.as_str(), "Morning");
        test_assert_eq!(applied.zones, [Some((ZoneRelay::Relay1, 15)), Some((ZoneRelay::Relay2, 5)), None, None]);
        test_assert_eq!(applied.status, Status::ACTIVE);

        test_assert_eq!(at_error(controller.exec(ScheduleController::AT_RESP))?, "No modify applied");
        Ok(())
    }))
}

fn test_set_invalid() -> Result<()> {
    with_schedules_restored(|controller| logged(|| {
        for raw in ["", "0", "x,mi,1", "0,xx,1", "0,mi", "0,mi,x", "0,mi,300", "0,mo,70000", "0,zn,1", "0,zn,1,x"] {
            test_assert_eq!(at_error(controller.set(ScheduleController::AT_RESP, Args { raw }))?, "InvalidArgs", "{raw:?} accepted");
        }
        Ok(())
    }))
}

fn test_set_out_of_range() -> Result<()> {
    with_schedules_restored(|controller| logged(|| {
        // Values the scheduler can never match must be refused, not stored
        for raw in ["0,mi,61", "0,hr,25", "0,dy,128", "0,dy,255", "0,mo,4096", "0,mo,65535", "0,zn,4,10", "0,zn,7,10", "0,st,2", "0,st,9"] {
            let accepted = controller.set(ScheduleController::AT_RESP, Args { raw }).is_ok();
            reset_staging();
            test_assert!(!accepted, "{raw:?} accepted");
        }
        Ok(())
    }))
}

fn test_set_bounds() -> Result<()> {
    with_schedules_restored(|controller| logged(|| {
        // NOT_SET (0) and the highest allowed value must both be accepted
        let last_relay = ZoneController::SIZE - 1;
        for raw in ["0,mi,0", "0,mi,60", "0,hr,0", "0,hr,24", "0,dy,0", "0,dy,127", "0,mo,0", "0,mo,4095", "0,st,0", "0,st,1", "0,zn,0,10"]
            .map(String::from)
            .into_iter()
            .chain([format!("0,zn,{last_relay},10")]) {
            let accepted = controller.set(ScheduleController::AT_RESP, Args { raw: &raw }).is_ok();
            reset_staging();
            test_assert!(accepted, "{raw:?} refused");
        }
        Ok(())
    }))
}

fn test_set_bad_index() -> Result<()> {
    with_schedules_restored(|controller| logged(|| {
        // Out of range indexes are refused by set and leave the staging area untouched
        for idx in [ScheduleController::SIZE, ScheduleController::SIZE + 1, usize::MAX] {
            test_assert_eq!(at_error(controller.set(ScheduleController::AT_RESP, Args { raw: &format!("{idx},mi,1") }))?, "InvalidArgs", "{idx} accepted");
            test_assert_eq!(at_error(controller.set(ScheduleController::AT_RESP, Args { raw: &format!("{idx},sv") }))?, "InvalidArgs", "{idx},sv accepted");
            test_assert!(!Schedule::is_modified(unsafe { &(*&raw const SCHEDULE_TMP).1 }), "{idx} staged");
        }
        test_assert_eq!(at_error(controller.exec(ScheduleController::AT_RESP))?, "No modify applied");
        Ok(())
    }))
}

fn test_exec_bad_index() -> Result<()> {
    with_schedules_restored(|controller| logged(|| {
        // set can no longer stage a bad index: force it to check the exec guard
        let mut staged = Schedule::new();
        staged.minute = 1;
        unsafe { SCHEDULE_TMP = (ScheduleController::SIZE, staged); }
        test_assert_eq!(at_error(controller.exec(ScheduleController::AT_RESP))?, "InvalidArgs");
        Ok(())
    }))
}

fn test_save_reload() -> Result<()> {
    with_schedules_restored(|controller| logged(|| {
        let expected = unsafe { (*&raw const SHARED).schedules };
        test_assert!(set(controller, "0,sv")?.is_empty());
        let reloaded: ScheduleController = deserialize_file(unsafe { &*&raw const MUTEX }, TAG, FS_CONFIG_DIR, ScheduleController::FILE_NAME)?;
        test_assert_eq!(reloaded.schedules, expected);
        Ok(())
    }))
}

fn test_test_form() -> Result<()> {
    test_assert!(!at_body(ScheduleController::shared().test(ScheduleController::AT_RESP))?.is_empty());
    Ok(())
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_day_map,
        test_month_map,
        test_is_modified,
        test_requires_login,
        test_set_query_exec,
        test_set_invalid,
        test_set_out_of_range,
        test_set_bounds,
        test_set_bad_index,
        test_exec_bad_index,
        test_save_reload,
        test_test_form,
    );
}
