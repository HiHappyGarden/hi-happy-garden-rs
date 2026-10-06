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


//! Session tests: `AT+SESS` login/logout and `AT+USR` user change, called
//! directly on the handlers.
//!
//! Logins use the user installed by `with_test_user` (memory only) and
//! every test ends logged out.

use alloc::format;
use alloc::string::String;

use at_parser_rs::Args;
use at_parser_rs::context::AtContext;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Error, Result};

use super::{Session, User, USER_LOGGED, USER_TMP};
use crate::apps::config::Config;
use crate::apps::parser::Parser;
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::test_helpers::{from_json, to_json, at_body, at_error, with_test_user, TEST_EMAIL, TEST_PASSWORD};
use crate::drivers::encrypt::EncryptGeneric;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::signal::Signal;

const TAG: &str = "SessionTests";

fn is_logged() -> bool {
    StatusSignal::get() & EventBits::from(StatusFlag::UserLogged) != 0
}

fn login(email: &str, password: &str) -> Result<String> {
    let session = Config::shared().get_session();
    let raw = format!("i,{email},{password}");
    at_body(session.set(Session::AT_RESP, Args { raw: &raw }))?;
    at_body(session.exec(Session::AT_RESP))
}

/// Runs `test`, then logs out whatever happened.
fn logged_out_after(test: impl FnOnce() -> Result<()>) -> Result<()> {
    let ret = test();
    Session::logout();
    ret
}

fn test_system_user() -> Result<()> {
    let mut config = Config::default();
    let system = config.get_session().users[0];
    test_assert!(!system.get_email().is_empty());
    test_assert_eq!(system.get_password().len(), 64);
    test_assert!(system.get_password().as_str().bytes().all(|b| b.is_ascii_hexdigit()));

    let mut session = Session::new();
    test_assert!(matches!(session.set_system_user("", "x"), Err(Error::Empty)));
    test_assert!(matches!(session.set_system_user("x", ""), Err(Error::Empty)));
    Ok(())
}

fn test_login_logout() -> Result<()> {
    with_test_user(|| logged_out_after(|| {
        test_assert!(!is_logged());
        test_assert_eq!(login(TEST_EMAIL, TEST_PASSWORD)?, TEST_EMAIL);
        test_assert!(is_logged());
        test_assert!(unsafe { (*&raw const USER_LOGGED).is_some() });

        let session = Config::shared().get_session();
        test_assert_eq!(at_body(session.query(Session::AT_RESP))?, TEST_EMAIL);

        at_body(session.set(Session::AT_RESP, Args { raw: "o" }))?;
        test_assert!(at_body(session.exec(Session::AT_RESP))?.is_empty());
        test_assert!(!is_logged());
        test_assert!(unsafe { (*&raw const USER_LOGGED).is_none() });
        test_assert_eq!(at_error(session.query(Session::AT_RESP))?, "InvalidArgs");
        Ok(())
    }))
}

fn test_logout_clears_source_flags() -> Result<()> {
    with_test_user(|| logged_out_after(|| {
        login(TEST_EMAIL, TEST_PASSWORD)?;
        StatusSignal::set(StatusFlag::UartCmd.into());
        Session::logout();
        let status = StatusSignal::get();
        for flag in [StatusFlag::UserLogged, StatusFlag::UartCmd, StatusFlag::MqttCmd, StatusFlag::SystemCmd] {
            test_assert!(status & EventBits::from(flag) == 0, "{flag:?} survived logout");
        }
        Ok(())
    }))
}

fn test_login_rejected() -> Result<()> {
    with_test_user(|| logged_out_after(|| {
        let session = Config::shared().get_session();
        for raw in [
            format!("i,{TEST_EMAIL},wrong"),
            format!("i,{TEST_EMAIL},"),
            format!("i,nobody@hhg.local,{TEST_PASSWORD}"),
        ] {
            at_body(session.set(Session::AT_RESP, Args { raw: &raw }))?;
            test_assert_eq!(at_error(session.exec(Session::AT_RESP))?, "InvalidArgs");
            test_assert!(!is_logged(), "logged in with {raw}");
            // The rejected credentials do not linger for the next exec
            test_assert!(unsafe { (*&raw const USER_TMP).email.is_empty() });
        }
        Ok(())
    }))
}

fn test_logout_requires_login() -> Result<()> {
    let session = Config::shared().get_session();
    at_body(session.set(Session::AT_RESP, Args { raw: "o" }))?;
    test_assert_eq!(at_error(session.exec(Session::AT_RESP))?, Parser::NOT_LOGGED_RESPONSE);
    Ok(())
}

fn test_set_invalid_args() -> Result<()> {
    let session = Config::shared().get_session();
    for raw in ["", "x", "i", "i,a@b"] {
        test_assert_eq!(at_error(session.set(Session::AT_RESP, Args { raw }))?, "InvalidArgs");
    }
    test_assert!(!at_body(session.test(Session::AT_RESP))?.is_empty());
    Ok(())
}

fn test_user_requires_login() -> Result<()> {
    let user = User::get_local();
    test_assert_eq!(at_error(user.query(User::AT_RESP))?, Parser::NOT_LOGGED_RESPONSE);
    test_assert_eq!(at_error(user.set(User::AT_RESP, Args { raw: "a@b,c" }))?, Parser::NOT_LOGGED_RESPONSE);
    test_assert_eq!(at_error(user.exec(User::AT_RESP))?, Parser::NOT_LOGGED_RESPONSE);
    Ok(())
}

fn test_user_change() -> Result<()> {
    with_test_user(|| logged_out_after(|| {
        login(TEST_EMAIL, TEST_PASSWORD)?;

        let user = User::get_local();
        test_assert!(at_body(user.set(User::AT_RESP, Args { raw: "new@hhg.local,n3w-pass" }))?.is_empty());
        test_assert_eq!(at_body(user.query(User::AT_RESP))?, "\"new@hhg.local\"");
        test_assert_eq!(at_body(user.exec(User::AT_RESP))?, "new@hhg.local");

        let local = Config::shared().get_session().get_user_local();
        test_assert_eq!(local.get_email().as_str(), "new@hhg.local");
        test_assert_eq!(local.get_password().as_str(), EncryptGeneric::get_sha256(b"n3w-pass")?.as_str());
        test_assert!(Config::shared().get_session().is_set_user_local());
        // The staging user is wiped once applied
        test_assert!(User::get_local().get_email().is_empty());

        // The new credentials work, the old ones no longer do
        Session::logout();
        test_assert_eq!(login("new@hhg.local", "n3w-pass")?, "new@hhg.local");
        Session::logout();
        test_assert!(login(TEST_EMAIL, TEST_PASSWORD).is_err());
        Ok(())
    }))
}

fn test_user_limits() -> Result<()> {
    with_test_user(|| logged_out_after(|| {
        login(TEST_EMAIL, TEST_PASSWORD)?;
        let user = User::get_local();
        let long = "a".repeat(33);
        test_assert_eq!(at_error(user.set(User::AT_RESP, Args { raw: &format!("{long},p") }))?, "email max len 32");
        test_assert_eq!(at_error(user.set(User::AT_RESP, Args { raw: &format!("a@b,{long}") }))?, "password max len 32");
        test_assert_eq!(at_error(user.set(User::AT_RESP, Args { raw: "a@b" }))?, "InvalidArgs");
        Ok(())
    }))
}

fn test_user_json() -> Result<()> {
    let mut user = User::default();
    user.set_email(TEST_EMAIL);
    user.set_password(EncryptGeneric::get_sha256(TEST_PASSWORD.as_bytes())?.as_str());

    let text = to_json(&user)?;
    test_assert!(text.contains(TEST_EMAIL));
    test_assert!(!text.contains(TEST_PASSWORD));

    let parsed: User = from_json(&text)?;
    test_assert_eq!(parsed.get_email(), user.get_email());
    test_assert_eq!(parsed.get_password(), user.get_password());
    test_assert!(!parsed.is_empty_passwd());

    let empty: User = from_json(r#"{"email":"a@b","password":""}"#)?;
    test_assert!(empty.is_empty_passwd());
    Ok(())
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_system_user,
        test_login_logout,
        test_logout_clears_source_flags,
        test_login_rejected,
        test_logout_requires_login,
        test_set_invalid_args,
        test_user_requires_login,
        test_user_change,
        test_user_limits,
        test_user_json,
    );
}
