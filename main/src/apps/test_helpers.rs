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


//! Shared helpers of the on-target application tests.

use alloc::format;
use alloc::string::String;

use at_parser_rs::{AtError, AtResult};
use osal_rs::utils::{Error, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::config::Config;
use crate::apps::parser::Parser;
use crate::apps::session::User;
use crate::apps::signals::error::ErrorSignal;
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::sprinkler::Sprinkler;
use crate::drivers::encrypt::EncryptGeneric;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;

/// Credentials of the user installed by [`with_test_user`]
pub(in crate::apps) const TEST_EMAIL: &str = "test@hhg.local";
pub(in crate::apps) const TEST_PASSWORD: &str = "t3st-p4ss";

/// Brings the application layer up the way `AppMain::init` does, without
/// the tasks (display, wifi, system led) the tests do not need.
pub(in crate::apps) fn setup() -> Result<()> {
    StatusSignal::init()?;
    ErrorSignal::init()?;
    Config::shared().init()?;
    Sprinkler::new().init()
}

/// Runs `test` with `UserLogged` set, as after a successful `AT+SESS`.
pub(in crate::apps) fn logged<R>(test: impl FnOnce() -> Result<R>) -> Result<R> {
    StatusSignal::set(StatusFlag::UserLogged.into());
    let ret = test();
    StatusSignal::clear(StatusFlag::UserLogged.into());
    ret
}

/// Runs `test` with [`TEST_EMAIL`] installed as local user, restoring the
/// real one afterwards. Memory only: `/etc/config.json` is not written.
pub(in crate::apps) fn with_test_user<R>(test: impl FnOnce() -> Result<R>) -> Result<R> {
    let session = Config::shared().get_session();
    let backup = session.get_user_local();

    let mut user = User::default();
    user.set_email(TEST_EMAIL);
    user.set_password(EncryptGeneric::get_sha256(TEST_PASSWORD.as_bytes())?.as_str());
    session.set_user(&user);

    let ret = test();
    Config::shared().get_session().set_user(&backup);
    ret
}

/// Body of a successful AT reply; an AT error becomes a test failure.
pub(in crate::apps) fn at_body(result: AtResult<'_, { Parser::CMD_SIZE }>) -> Result<String> {
    match result {
        Ok((_, body)) => Ok(String::from(body.as_str())),
        Err((_, e)) => Err(Error::UnhandledOwned(format!("unexpected AT error: {e:?}"))),
    }
}

/// Text of a failed AT reply (the message, or the variant name); a
/// successful reply becomes a test failure.
pub(in crate::apps) fn at_error(result: AtResult<'_, { Parser::CMD_SIZE }>) -> Result<String> {
    match result {
        Ok((_, body)) => Err(Error::UnhandledOwned(format!("expected AT error, got OK: {:?}", body.as_str()))),
        Err((_, AtError::Unhandled(msg))) => Ok(String::from(msg)),
        Err((_, AtError::UnhandledOwned(msg))) => Ok(msg),
        Err((_, e)) => Ok(format!("{e:?}")),
    }
}

/// `cjson_binding::to_json` with the error converted for `?` in tests.
pub(in crate::apps) fn to_json<T: Serialize>(value: &T) -> Result<String> {
    cjson_binding::to_json(value).map_err(|e| Error::UnhandledOwned(format!("to_json: {e}")))
}

/// `cjson_binding::from_json` with the error converted for `?` in tests.
pub(in crate::apps) fn from_json<T: Deserialize + Default>(json: &str) -> Result<T> {
    cjson_binding::from_json(&String::from(json)).map_err(|e| Error::UnhandledOwned(format!("from_json: {e}")))
}
