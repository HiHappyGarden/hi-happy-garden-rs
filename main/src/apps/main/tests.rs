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


//! `AppMain` tests. The FSM thread itself is not started (it would take
//! over the display, the UART and the relays); its steps are tested where
//! they are plain functions.

use osal_rs::utils::{Bytes, Result};

use super::AppMain;
use crate::apps::config::Config;
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::signal::Signal;

const TAG: &str = "AppMainTests";

/// Runs `check_config` from `CheckConfig` with `serial`, restoring the serial
/// and the status bits it touches.
fn check_config_with(serial: &str) -> Result<(StatusFlag, StatusFlag, u32)> {
    let config = Config::shared();
    let backup = config.get_serial();
    config.set_serial(&Bytes::from_str(serial));

    let mut current = StatusFlag::CheckConfig;
    let mut old = StatusFlag::EnableDisplay;
    StatusSignal::set(StatusFlag::CheckConfig.into());
    AppMain::check_config(config, &mut current, &mut old);
    let status = StatusSignal::get();

    StatusSignal::clear(u32::from(StatusFlag::CheckConfig) | u32::from(StatusFlag::EnableWifi));
    config.set_serial(&backup);
    Ok((current, old, status))
}

fn test_check_config_with_serial() -> Result<()> {
    let (current, old, status) = check_config_with("HHG-TEST")?;
    test_assert_eq!(current, StatusFlag::EnableWifi);
    test_assert_eq!(old, StatusFlag::CheckConfig);
    test_assert!(StatusFlag::EnableWifi.check_signal(status));
    test_assert!(!StatusFlag::CheckConfig.check_signal(status));
    Ok(())
}

fn test_check_config_without_serial() -> Result<()> {
    // No serial yet: the FSM waits in CheckConfig (the wizard sets it)
    let (current, old, _) = check_config_with("")?;
    test_assert_eq!(current, StatusFlag::CheckConfig);
    test_assert_eq!(old, StatusFlag::EnableDisplay);
    Ok(())
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_check_config_with_serial,
        test_check_config_without_serial,
    );
}
