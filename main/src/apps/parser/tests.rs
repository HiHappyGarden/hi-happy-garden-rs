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


//! Parser tests: command dispatch and the full UART path.
//!
//! The end-to-end tests start the real parser task and feed it one byte at
//! a time through `on_receive`, exactly like the UART RX interrupt does;
//! the replies are captured by [`MockChannel`] in place of the UART TX.

use core::time::Duration;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use at_parser_rs::AtError;
use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;
use osal_rs::os::{Mutex, MutexFn, System};
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Error, Result};

use super::Parser;
use crate::apps::config::{Config, DaylightSavingTime, NtpConfig, WifiConfig};
use crate::apps::session::{Session, User};
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::sprinkler::schedule::ScheduleController;
use crate::apps::sprinkler::zone::ZoneController;
use crate::apps::system_handler::SystemHandler;
use crate::apps::test_helpers::{with_test_user, TEST_EMAIL, TEST_PASSWORD};
use crate::drivers::error::HardwareErrorSignal;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::rx_tx::{OnReceive, SetTransmit, Source};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;

const TAG: &str = "ParserTests";

const REPLY_TIMEOUT_MS: u32 = 1_000;

static mut OUTPUT: Option<Mutex<Vec<u8>>> = None;

/// Stands in for the UART TX: collects everything the parser transmits.
struct MockChannel;

impl SetTransmit for MockChannel {
    fn transmit(&self, data: &[u8]) -> usize {
        if let Some(output) = unsafe { &*&raw const OUTPUT } {
            if let Ok(mut output) = output.lock() {
                output.extend_from_slice(data);
            }
        }
        data.len()
    }
}

static MOCK_CHANNEL: MockChannel = MockChannel;

/// Feeds `line` + CRLF byte by byte, pausing every few bytes so the parser
/// task drains its 256 byte queue as it would at UART speed.
fn post(line: &str) -> Result<()> {
    let parser = Parser::shared();
    for (idx, byte) in line.bytes().chain(*b"\r\n").enumerate() {
        parser.on_receive(Source::Uart, &[byte])?;
        if idx % 32 == 31 {
            System::delay_with_to_tick(Duration::from_millis(5));
        }
    }
    Ok(())
}

/// Sends `line` and waits for one reply line.
fn send(line: &str) -> Result<String> {
    post(line)?;

    let output = unsafe { &*&raw const OUTPUT }.as_ref().ok_or(Error::NullPtr)?;
    for _ in 0..REPLY_TIMEOUT_MS / 10 {
        System::delay_with_to_tick(Duration::from_millis(10));
        let mut output = output.lock()?;
        if output.ends_with(b"\r\n") {
            let reply = String::from_utf8_lossy(&output[..output.len() - 2]).into_owned();
            output.clear();
            return Ok(reply);
        }
    }
    Err(Error::UnhandledOwned(format!("no reply to {line:?}")))
}

fn is_set(flag: StatusFlag) -> bool {
    StatusSignal::get() & EventBits::from(flag) != 0
}

fn setup() -> Result<()> {
    unsafe {
        OUTPUT = Some(Mutex::new(Vec::new()));
    }
    Parser::set_uart_transmit(&MOCK_CHANNEL);
    Parser::shared().init()
}

/// The same command table the parser task builds.
fn with_parser(test: impl FnOnce(&mut AtParser<dyn AtContext<{ Parser::CMD_SIZE }>, { Parser::CMD_SIZE }>) -> Result<()>) -> Result<()> {
    let mut parser: AtParser<dyn AtContext<{ Parser::CMD_SIZE }>, { Parser::CMD_SIZE }> = AtParser::new();
    let commands: &mut [(&str, &str, &mut dyn AtContext<{ Parser::CMD_SIZE }>)] = &mut [
        (Config::AT_CMD, Config::AT_RESP, Config::shared()),
        (Session::AT_CMD, Session::AT_RESP, Config::shared().get_session()),
        (User::AT_CMD, User::AT_RESP, User::get_local()),
        (SystemHandler::AT_CMD, SystemHandler::AT_RESP, SystemHandler::get()),
        (DaylightSavingTime::AT_CMD, DaylightSavingTime::AT_RESP, Config::shared().get_daylight_saving_time()),
        (WifiConfig::AT_CMD, WifiConfig::AT_RESP, Config::shared().get_wifi_config()),
        (NtpConfig::AT_CMD, NtpConfig::AT_RESP, Config::shared().get_ntp_config_mut()),
        (ScheduleController::AT_CMD, ScheduleController::AT_RESP, ScheduleController::shared()),
        (ZoneController::AT_CMD, ZoneController::AT_RESP, ZoneController::shared()),
    ];
    parser.set_commands(commands);
    test(&mut parser)
}

fn test_dispatch_forms() -> Result<()> {
    with_parser(|parser| {
        // =? -> test
        let (prefix, body) = parser.execute("AT+SYS=?").map_err(|_| Error::Unhandled("AT+SYS=? failed"))?;
        test_assert_eq!(prefix, SystemHandler::AT_RESP);
        test_assert_eq!(body.as_str(), "<rs|fr|hwe|e|s>");

        // ? -> query
        let (_, body) = parser.execute("  AT+SYS?  ").map_err(|_| Error::Unhandled("AT+SYS? failed"))?;
        let expected = format!("{},{},{}", HardwareErrorSignal::get(), crate::apps::signals::error::ErrorSignal::get(), StatusSignal::get());
        test_assert_eq!(body.as_str(), expected);

        // = -> set, routed to the right module
        let ret = parser.execute("AT+CNF=tz,0");
        test_assert!(matches!(ret, Err((Config::AT_RESP, AtError::Unhandled(Parser::NOT_LOGGED_RESPONSE)))));
        Ok(())
    })
}

fn test_dispatch_every_command() -> Result<()> {
    with_parser(|parser| {
        for cmd in [Config::AT_CMD, Session::AT_CMD, User::AT_CMD, SystemHandler::AT_CMD, DaylightSavingTime::AT_CMD,
                    WifiConfig::AT_CMD, NtpConfig::AT_CMD, ScheduleController::AT_CMD, ZoneController::AT_CMD] {
            let line = format!("{cmd}=?");
            let ret = parser.execute(&line);
            test_assert!(matches!(&ret, Ok((_, body)) if !body.is_empty()), "{line} has no test form");
        }
        Ok(())
    })
}

fn test_dispatch_unknown() -> Result<()> {
    with_parser(|parser| {
        for line in ["AT+NOPE", "AT+NOPE?", "AT+CN=?", "AT+CNFX?", "", "AT"] {
            test_assert!(matches!(parser.execute(line), Err(("", AtError::UnknownCommand))), "{line:?} dispatched");
        }
        Ok(())
    })
}

fn test_dispatch_not_supported() -> Result<()> {
    with_parser(|parser| {
        // SystemHandler has no exec form, Config has none either
        test_assert!(matches!(parser.execute("AT+SYS"), Err((SystemHandler::AT_RESP, AtError::NotSupported))));
        test_assert!(matches!(parser.execute("AT+CNF"), Err((Config::AT_RESP, AtError::NotSupported))));
        Ok(())
    })
}

fn test_uart_test_form() -> Result<()> {
    test_assert_eq!(send("AT+SYS=?")?, "+SYS: <rs|fr|hwe|e|s>");
    test_assert_eq!(send("AT+CNF=?")?, "+CNF: sn,<value> | tz,<value> | sv");
    Ok(())
}

fn test_uart_unknown_command() -> Result<()> {
    test_assert_eq!(send("AT+NOPE")?, "KO");
    test_assert_eq!(send("garbage")?, "KO");
    Ok(())
}

fn test_uart_not_logged() -> Result<()> {
    test_assert_eq!(send("AT+CNF=tz,0")?, format!("+CNF: {}", Parser::NOT_LOGGED_RESPONSE));
    test_assert_eq!(send("AT+SYS=s")?, format!("+SYS: {}", Parser::NOT_LOGGED_RESPONSE));
    test_assert!(!is_set(StatusFlag::UartCmd));
    Ok(())
}

fn test_uart_session() -> Result<()> {
    with_test_user(|| {
        let ret = (|| {
            test_assert_eq!(send(&format!("AT+SESS=i,{TEST_EMAIL},{TEST_PASSWORD}"))?, "+SESS: OK");
            test_assert_eq!(send("AT+SESS")?, format!("+SESS: {TEST_EMAIL}"));
            test_assert!(is_set(StatusFlag::UserLogged));
            // The session is bound to the channel it was opened on
            test_assert!(is_set(StatusFlag::UartCmd));

            test_assert_eq!(send("AT+SESS?")?, format!("+SESS: {TEST_EMAIL}"));
            let reply = send("AT+SYS=s")?;
            test_assert!(reply.starts_with("+SYS: ") && reply[6..].parse::<u32>().is_ok(), "status reply {reply:?}");

            test_assert_eq!(send("AT+SESS=o")?, "+SESS: OK");
            test_assert_eq!(send("AT+SESS")?, "+SESS: OK");
            test_assert!(!is_set(StatusFlag::UserLogged));
            test_assert!(!is_set(StatusFlag::UartCmd));
            test_assert_eq!(send("AT+SYS=s")?, format!("+SYS: {}", Parser::NOT_LOGGED_RESPONSE));
            Ok(())
        })();
        if is_set(StatusFlag::UserLogged) {
            let _ = send("AT+SESS=o");
            let _ = send("AT+SESS");
        }
        ret
    })
}

fn test_uart_bad_login() -> Result<()> {
    with_test_user(|| {
        test_assert_eq!(send(&format!("AT+SESS=i,{TEST_EMAIL},wrong"))?, "+SESS: OK");
        test_assert_eq!(send("AT+SESS")?, "+SESS: KO");
        test_assert!(!is_set(StatusFlag::UserLogged));
        test_assert!(!is_set(StatusFlag::UartCmd));
        Ok(())
    })
}

fn test_uart_long_line() -> Result<()> {
    // Longer than the 256 byte RX buffer: dropped, and the next command still works
    let long = format!("AT+CNF=sn,{}", "x".repeat(300));
    // The 256 bytes before the overflow are dropped, the tail is an unknown command
    test_assert_eq!(send(&long)?, "KO");
    test_assert_eq!(send("AT+SYS=?")?, "+SYS: <rs|fr|hwe|e|s>");
    Ok(())
}

fn test_uart_two_lines_in_one_buffer() -> Result<()> {
    // Bug #9: one on_receive carries one command. A second line in the same data has no
    // source: it must be discarded (it used to make the parser task panic)
    Parser::shared().on_receive(Source::Uart, b"AT+SYS=?\r\nAT+SYS=?\r\n")?;
    System::delay_with_to_tick(Duration::from_millis(200));

    let output = unsafe { &*&raw const OUTPUT }.as_ref().ok_or(Error::NullPtr)?;
    let replies = {
        let mut output = output.lock()?;
        let replies = String::from_utf8_lossy(&output).into_owned();
        output.clear();
        replies
    };
    test_assert_eq!(replies, "+SYS: <rs|fr|hwe|e|s>\r\n");

    // The parser is still alive and the source is free again
    test_assert_eq!(send("AT+SYS=?")?, "+SYS: <rs|fr|hwe|e|s>");
    Ok(())
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_dispatch_forms,
        test_dispatch_every_command,
        test_dispatch_unknown,
        test_dispatch_not_supported,
    );

    stats.record(TAG, "setup", setup());

    run_tests!(TAG, stats;
        test_uart_test_form,
        test_uart_unknown_command,
        test_uart_not_logged,
        test_uart_session,
        test_uart_bad_login,
        test_uart_long_line,
        test_uart_two_lines_in_one_buffer,
    );
}
