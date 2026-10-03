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


//! Zone tests: relay mapping and the `AT+ZN` handler.
//!
//! Each AT test restores the shared zones and the staging area. `sv` is
//! sent once, by `test_save_reload`, on the restored (unchanged) zones.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use at_parser_rs::Args;
use at_parser_rs::context::AtContext;
use osal_rs::utils::Result;

use super::{MUTEX, SHARED, ZONE_TMP, Zone, ZoneController, ZoneRelay};
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::parser::Parser;
use crate::apps::test_helpers::{at_body, at_error, logged};
use crate::apps::utils::deserialize_file;
use crate::drivers::platform::{FS_CONFIG_DIR, GpioPeripheral};
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};

const TAG: &str = "ZoneTests";

const RELAYS: [ZoneRelay; 4] = [ZoneRelay::Relay0, ZoneRelay::Relay1, ZoneRelay::Relay2, ZoneRelay::Relay3];

fn zones() -> [Zone; ZoneController::SIZE] {
    unsafe { (*&raw const SHARED).0 }
}

fn zone(relay: ZoneRelay) -> Result<Zone> {
    zones().into_iter().find(|zone| zone.zone_relay == relay)
        .ok_or(osal_rs::utils::Error::UnhandledOwned(format!("no zone on {relay}")))
}

/// Runs `test` on the shared zones, then puts them and the staging area back.
fn with_zones_restored(test: impl FnOnce(&'static mut ZoneController) -> Result<()>) -> Result<()> {
    let backup = *ZoneController::shared();
    unsafe { ZONE_TMP = Zone::new(ZoneRelay::Relay0); }
    let ret = test(ZoneController::shared());
    *ZoneController::shared() = backup;
    unsafe { ZONE_TMP = Zone::new(ZoneRelay::Relay0); }
    ret
}

fn set(controller: &mut ZoneController, raw: &str) -> Result<String> {
    at_body(controller.set(ZoneController::AT_RESP, Args { raw }))
}

fn test_relay_conversions() -> Result<()> {
    let gpio = [GpioPeripheral::Relay0, GpioPeripheral::Relay1, GpioPeripheral::Relay2, GpioPeripheral::Relay3];
    for (idx, relay) in RELAYS.into_iter().enumerate() {
        test_assert_eq!(u8::from(relay), idx as u8);
        test_assert_eq!(ZoneRelay::from(idx as u8), relay);
        test_assert_eq!(GpioPeripheral::from(relay), gpio[idx]);
        test_assert_eq!(format!("{relay}"), format!("Relay {idx}"));
    }
    Ok(())
}

fn test_is_modified() -> Result<()> {
    test_assert!(!Zone::is_modified(&Zone::new(ZoneRelay::Relay0)));
    let mut zone = Zone::new(ZoneRelay::Relay0);
    zone.weight = 1;
    test_assert!(Zone::is_modified(&zone));
    test_assert!(Zone::is_modified(&Zone::new(ZoneRelay::Relay2)));
    Ok(())
}

fn test_zones_cover_all_relays() -> Result<()> {
    // One zone per relay, otherwise a relay can never be driven
    let relays: Vec<ZoneRelay> = zones().iter().map(|zone| zone.zone_relay).collect();
    for relay in RELAYS {
        test_assert!(relays.contains(&relay), "no zone on {relay}, zones: {relays:?}");
    }
    Ok(())
}

fn test_zones_have_description() -> Result<()> {
    for zone in zones() {
        test_assert!(!zone.description.is_empty(), "zone on {} without description", zone.zone_relay);
        test_assert!(zone.description.as_str().chars().all(|c| !c.is_control()), "zone on {} has control chars", zone.zone_relay);
    }
    Ok(())
}

fn test_requires_login() -> Result<()> {
    with_zones_restored(|controller| {
        test_assert_eq!(at_error(controller.query(ZoneController::AT_RESP))?, Parser::NOT_LOGGED_RESPONSE);
        test_assert_eq!(at_error(controller.set(ZoneController::AT_RESP, Args { raw: "0,wt,1" }))?, Parser::NOT_LOGGED_RESPONSE);
        test_assert_eq!(at_error(controller.exec(ZoneController::AT_RESP))?, Parser::NOT_LOGGED_RESPONSE);
        Ok(())
    })
}

fn test_query() -> Result<()> {
    logged(|| {
        let body = at_body(ZoneController::shared().query(ZoneController::AT_RESP))?;
        let expected: String = zones().iter()
            .map(|zone| format!("{},{},\"{}\"\r\n", u8::from(zone.zone_relay), zone.weight, zone.description.as_str()))
            .collect();
        test_assert_eq!(body, expected);
        Ok(())
    })
}

fn test_set_exec() -> Result<()> {
    with_zones_restored(|controller| logged(|| {
        test_assert!(set(controller, "0,ds,Lawn")?.is_empty());
        test_assert!(at_body(controller.exec(ZoneController::AT_RESP))?.is_empty());
        test_assert_eq!(zone(ZoneRelay::Relay0)?.description.as_str(), "Lawn");

        let weight = zone(ZoneRelay::Relay0)?.weight.wrapping_add(1);
        test_assert!(set(controller, &format!("0,wt,{weight}"))?.is_empty());
        // Staged only: applied by exec
        test_assert!(zone(ZoneRelay::Relay0)?.weight != weight, "set applied before exec");
        test_assert!(at_body(controller.exec(ZoneController::AT_RESP))?.is_empty());
        test_assert_eq!(zone(ZoneRelay::Relay0)?.weight, weight);
        test_assert_eq!(zone(ZoneRelay::Relay0)?.description.as_str(), "Lawn");

        // Nothing left staged after an exec
        test_assert_eq!(at_error(controller.exec(ZoneController::AT_RESP))?, "No modify applied");
        Ok(())
    }))
}

fn test_set_many_then_exec() -> Result<()> {
    with_zones_restored(|controller| logged(|| {
        // Stage two fields, apply both with one exec
        set(controller, "0,wt,5")?;
        set(controller, "0,ds,Orchard")?;
        at_body(controller.exec(ZoneController::AT_RESP))?;
        let zone = zone(ZoneRelay::Relay0)?;
        test_assert_eq!(zone.description.as_str(), "Orchard");
        test_assert_eq!(zone.weight, 5, "weight staged before the description was lost");
        Ok(())
    }))
}

fn test_set_invalid() -> Result<()> {
    with_zones_restored(|controller| logged(|| {
        for raw in ["", "0", "x,wt,1", "0,xx,1", "0,wt", "0,wt,300", "0,wt,x"] {
            test_assert_eq!(at_error(controller.set(ZoneController::AT_RESP, Args { raw }))?, "InvalidArgs", "{raw:?} accepted");
        }
        let long = "d".repeat(DISPLAY_INPUT_MAX_SIZE + 1);
        let raw = format!("0,ds,{long}");
        test_assert_eq!(at_error(controller.set(ZoneController::AT_RESP, Args { raw: &raw }))?, "description max len exceeded");
        Ok(())
    }))
}

fn test_set_unknown_relay() -> Result<()> {
    with_zones_restored(|controller| logged(|| {
        let before = zone(ZoneRelay::Relay0)?;
        let ret = controller.set(ZoneController::AT_RESP, Args { raw: "7,ds,Ghost" });
        let accepted = ret.is_ok();
        if accepted {
            let _ = controller.exec(ZoneController::AT_RESP);
        }
        test_assert!(!accepted, "relay 7 accepted");
        test_assert_eq!(zone(ZoneRelay::Relay0)?, before);
        Ok(())
    }))
}

fn test_save_reload() -> Result<()> {
    with_zones_restored(|controller| logged(|| {
        let expected = zones();
        test_assert!(set(controller, "0,sv")?.is_empty());
        let reloaded: ZoneController = deserialize_file(unsafe { &*&raw const MUTEX }, TAG, FS_CONFIG_DIR, ZoneController::FILE_NAME)?;
        test_assert_eq!(reloaded.0, expected);
        Ok(())
    }))
}

fn test_screen_selections() -> Result<()> {
    let selections = ZoneController::new_selections();
    for (selection, zone) in selections.iter().zip(zones().iter()) {
        test_assert_eq!(selection.0.as_str(), zone.description.as_str());
        test_assert!(!selection.1);
    }
    Ok(())
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_relay_conversions,
        test_is_modified,
        test_zones_cover_all_relays,
        test_zones_have_description,
        test_requires_login,
        test_query,
        test_set_exec,
        test_set_many_then_exec,
        test_set_invalid,
        test_set_unknown_relay,
        test_save_reload,
        test_screen_selections,
    );
}
