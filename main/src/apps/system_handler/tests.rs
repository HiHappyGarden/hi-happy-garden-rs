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


//! `AT+SYS` tests. `rs` (reboot) and `fr` (factory reset) end in a reset
//! and are deliberately not exercised.

use alloc::format;

use at_parser_rs::Args;
use at_parser_rs::context::AtContext;
use osal_rs::utils::Result;

use super::SystemHandler;
use crate::apps::parser::Parser;
use crate::apps::signals::error::ErrorSignal;
use crate::apps::signals::status::StatusSignal;
use crate::apps::test_helpers::{at_body, at_error, logged};
use crate::drivers::error::HardwareErrorSignal;
use crate::tests::{TestStats, run_tests, test_assert_eq};
use crate::traits::signal::Signal;

const TAG: &str = "SystemHandlerTests";

fn test_query() -> Result<()> {
    let body = at_body(SystemHandler::get().query(SystemHandler::AT_RESP))?;
    test_assert_eq!(body, format!("{},{},{}", HardwareErrorSignal::get(), ErrorSignal::get(), StatusSignal::get()));
    Ok(())
}

fn test_requires_login() -> Result<()> {
    for raw in ["s", "e", "hwe", "rs", "fr"] {
        test_assert_eq!(at_error(SystemHandler::get().set(SystemHandler::AT_RESP, Args { raw }))?, Parser::NOT_LOGGED_RESPONSE);
    }
    Ok(())
}

fn test_read_signals() -> Result<()> {
    logged(|| {
        let handler = SystemHandler::get();
        test_assert_eq!(at_body(handler.set(SystemHandler::AT_RESP, Args { raw: "hwe" }))?, format!("{}", HardwareErrorSignal::get()));
        test_assert_eq!(at_body(handler.set(SystemHandler::AT_RESP, Args { raw: "e" }))?, format!("{}", ErrorSignal::get()));
        test_assert_eq!(at_body(handler.set(SystemHandler::AT_RESP, Args { raw: "s" }))?, format!("{}", StatusSignal::get()));
        for raw in ["", "x", "S"] {
            test_assert_eq!(at_error(handler.set(SystemHandler::AT_RESP, Args { raw }))?, "InvalidArgs");
        }
        Ok(())
    })
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_query,
        test_requires_login,
        test_read_signals,
    );
}
