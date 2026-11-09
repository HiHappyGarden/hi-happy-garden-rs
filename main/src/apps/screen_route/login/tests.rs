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

//! Login screen tests: the credentials typed on the display against the
//! users saved by the wizard and the user screen (`User::from_credentials`).
//! Input editing needs the encoder and is not covered here: the tests call
//! `login_with` with the values the inputs would hold.

use alloc::string::String;

use at_parser_rs::Args;
use at_parser_rs::context::AtContext;
use osal_rs::os::types::EventBits;
use osal_rs::utils::Result;

use super::ScreenLogin;
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::config_shared;
use crate::apps::session::{Session, User};
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::test_helpers::{TEST_EMAIL, TEST_PASSWORD, with_test_user};
use crate::tests::{TestStats, run_tests, test_assert};
use crate::traits::signal::Signal;

const TAG: &str = "LoginTests";

fn is_logged() -> bool {
    StatusSignal::get() & EventBits::from(StatusFlag::UserLogged) != 0
}

/// `AT+SESS=o` then exec; a failure only means nobody was logged.
fn logout() {
    let mut config = config_shared!();
    let session = config.get_session();
    let _ = session.set(Session::AT_RESP, Args { raw: "o" });
    let _ = session.exec(Session::AT_RESP);
}

/// Runs `test` with `email`/`password` saved as the wizard does, then
/// logs out and restores the real local user. Memory only.
fn with_user(email: &str, password: &str, test: impl FnOnce() -> Result<()>) -> Result<()> {
    let user = User::from_credentials(email, password)?;
    // Guard released before `test`, which locks `Config` again
    let backup = {
        let mut config = config_shared!();
        let session = config.get_session();
        let backup = session.get_user_local();
        session.set_user(&user);
        backup
    };
    let ret = test();
    logout();
    config_shared!().get_session().set_user(&backup);
    ret
}

fn test_login_wizard_user() -> Result<()> {
    with_test_user(|| {
        let ret = (|| {
            test_assert!(!is_logged());
            test_assert!(ScreenLogin::new().login_with(TEST_EMAIL, TEST_PASSWORD)?, "user saved by the wizard rejected");
            test_assert!(is_logged());
            Ok(())
        })();
        logout();
        ret
    })
}

fn test_login_rejected() -> Result<()> {
    with_test_user(|| {
        let screen = ScreenLogin::new();
        for (email, password) in [
            (TEST_EMAIL, "wrong"),
            (TEST_EMAIL, ""),
            ("nobody@hhg.local", TEST_PASSWORD),
            ("", TEST_PASSWORD),
        ] {
            test_assert!(!screen.login_with(email, password)?, "logged in with {email:?}/{password:?}");
            test_assert!(!is_logged(), "logged flag set by {email:?}/{password:?}");
        }
        Ok(())
    })
}

/// The wizard accepts an empty password: it must log in with an empty one.
fn test_login_empty_password() -> Result<()> {
    with_user("empty@hhg.local", "", || {
        let screen = ScreenLogin::new();
        test_assert!(!screen.login_with("empty@hhg.local", "x")?);
        test_assert!(screen.login_with("empty@hhg.local", "")?, "empty password rejected");
        Ok(())
    })
}

/// Quotes, backslashes and commas survive the `AT+SESS` quoting.
fn test_login_special_chars() -> Result<()> {
    const EMAIL: &str = "a\"b\\c,d@hhg.local";
    const PASSWORD: &str = "p\"a,s\\s";
    with_user(EMAIL, PASSWORD, || {
        test_assert!(ScreenLogin::new().login_with(EMAIL, PASSWORD)?, "escaped credentials rejected");
        Ok(())
    })
}

/// Longest inputs, every byte escaped: the raw command still fits.
fn test_login_max_len() -> Result<()> {
    let email: String = "\"".repeat(DISPLAY_INPUT_MAX_SIZE);
    let password: String = "\\".repeat(DISPLAY_INPUT_MAX_SIZE);
    with_user(&email, &password, || {
        test_assert!(ScreenLogin::new().login_with(&email, &password)?, "max length credentials rejected");
        Ok(())
    })
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_login_wizard_user,
        test_login_rejected,
        test_login_empty_password,
        test_login_special_chars,
        test_login_max_len,
    );
}
