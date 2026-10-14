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

use core::sync::atomic::{AtomicI64, Ordering};

use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use at_parser_rs::{AtError, AtResult};
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Error, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::config::Config;
use crate::apps::parser::Parser;
use crate::apps::session::User;
use crate::apps::signals::display::{DisplayFlag, DisplaySignal};
use crate::apps::signals::error::ErrorSignal;
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::sprinkler::Sprinkler;
use crate::drivers::encrypt::EncryptGeneric;
use crate::traits::lcd_display::{LCDDisplayFn, LCDWriteMode};
use crate::traits::rtc::RTC;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;

/// Credentials of the user installed by [`with_test_user`]
pub(in crate::apps) const TEST_EMAIL: &str = "test@hhg.local";
pub(in crate::apps) const TEST_PASSWORD: &str = "t3st-p4ss";

/// Brings the application layer up the way `AppMain::init` does, without
/// the tasks (display, wifi, system led) the tests do not need. The display
/// signal is created anyway: screens raise `Draw` on it.
pub(in crate::apps) fn setup() -> Result<()> {
    StatusSignal::init()?;
    ErrorSignal::init()?;
    DisplaySignal::init()?;
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

/// Runs `test`, then clears the `Draw` bit of the display signal if it
/// was not set before: screens raise it on every transition.
pub(in crate::apps) fn with_draw_restored<R>(test: impl FnOnce() -> Result<R>) -> Result<R> {
    let was_set = DisplaySignal::get() & DRAW != 0;
    let ret = test();
    if !was_set {
        DisplaySignal::clear(DRAW);
    }
    ret
}

pub(in crate::apps) const DRAW: EventBits = DisplayFlag::Draw as EventBits;

/// One text drawn by a widget: `draw_str` and `draw_bytes` both end up here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::apps) struct DrawnText {
    pub(in crate::apps) text: Vec<u8>,
    pub(in crate::apps) x: u8,
    pub(in crate::apps) y: u8,
}

/// In-memory LCD with the SH1106 geometry: records what the widgets draw,
/// so their output can be checked without a panel.
#[derive(Clone, Default)]
pub(in crate::apps) struct MockLcd {
    pub(in crate::apps) texts: Vec<DrawnText>,
    pub(in crate::apps) rects: Vec<(u8, u8, u8, u8, LCDWriteMode)>,
    pub(in crate::apps) bitmaps: Vec<(u8, u8, u8, u8, Vec<u8>)>,
    /// Overrides the panel height, to exercise geometry edge cases.
    pub(in crate::apps) height: Option<u8>,
}

impl MockLcd {
    pub(in crate::apps) const WIDTH: u8 = 132;
    pub(in crate::apps) const HEIGHT: u8 = 64;

    pub(in crate::apps) fn new() -> Self {
        Self::default()
    }

    /// Texts drawn at row `y`, oldest first.
    pub(in crate::apps) fn texts_at(&self, y: u8) -> Vec<String> {
        self.texts
            .iter()
            .filter(|t| t.y == y)
            .map(|t| String::from_utf8_lossy(&t.text).into_owned())
            .collect()
    }

    /// Last text drawn at row `y`, or an empty string.
    pub(in crate::apps) fn last_text_at(&self, y: u8) -> String {
        self.texts_at(y).pop().unwrap_or_default()
    }

    pub(in crate::apps) fn reset(&mut self) {
        self.texts.clear();
        self.rects.clear();
        self.bitmaps.clear();
    }
}

impl LCDDisplayFn for MockLcd {
    fn get_size(&self) -> (u8, u8) {
        (Self::WIDTH, self.height.unwrap_or(Self::HEIGHT))
    }

    fn get_visible_size(&self) -> (u8, u8) {
        let (width, height) = self.get_size();
        (width - 4, height)
    }

    fn draw(&mut self) -> Result<()> {
        Ok(())
    }

    fn clear(&mut self) {
        self.reset();
    }

    fn draw_pixel(&mut self, _: u8, _: u8, _: LCDWriteMode) -> Result<()> {
        Ok(())
    }

    fn draw_bitmap_image(&mut self, x: u8, y: u8, width: u8, height: u8, image: &[u8], _: LCDWriteMode) -> Result<()> {
        self.bitmaps.push((x, y, width, height, image.to_vec()));
        Ok(())
    }

    fn draw_rect(&mut self, x: u8, y: u8, width: u8, height: u8, write_mode: LCDWriteMode) -> Result<()> {
        self.rects.push((x, y, width, height, write_mode));
        Ok(())
    }

    fn draw_char(&mut self, c: char, x: u8, y: u8, _: &[u8]) -> Result<()> {
        let mut buf = [0u8; 4];
        self.texts.push(DrawnText { text: c.encode_utf8(&mut buf).as_bytes().to_vec(), x, y });
        Ok(())
    }

    fn draw_str(&mut self, str: &str, x: u8, y: u8, _: &[u8]) -> Result<()> {
        self.texts.push(DrawnText { text: str.as_bytes().to_vec(), x, y });
        Ok(())
    }

    fn draw_bytes(&mut self, bytes: &[u8], x: u8, y: u8, _: &[u8]) -> Result<()> {
        self.texts.push(DrawnText { text: bytes.to_vec(), x, y });
        Ok(())
    }

    fn invert_orientation(&mut self) -> Result<()> {
        Ok(())
    }

    fn set_contrast(&self, _: u8) -> Result<()> {
        Ok(())
    }

    fn turn_off(&mut self) -> Result<()> {
        Ok(())
    }

    fn turn_on(&mut self) -> Result<()> {
        Ok(())
    }
}

/// RTC frozen at a timestamp, for the widgets that read the time.
pub(in crate::apps) struct MockRtc(pub(in crate::apps) AtomicI64);

impl RTC for MockRtc {
    fn set_timestamp(&self, timestamp: i64) -> Result<()> {
        self.0.store(timestamp, Ordering::Relaxed);
        Ok(())
    }

    fn get_timestamp(&self) -> Result<i64> {
        Ok(self.0.load(Ordering::Relaxed))
    }

    fn is_to_synch(&self) -> bool {
        false
    }
}

/// A [`MockRtc`] shared the way the display task receives the real one.
pub(in crate::apps) fn mock_rtc(timestamp: i64) -> Arc<Mutex<dyn RTC + 'static>> {
    Mutex::new_arc(MockRtc(AtomicI64::new(timestamp)))
}

/// `cjson_binding::to_json` with the error converted for `?` in tests.
pub(in crate::apps) fn to_json<T: Serialize>(value: &T) -> Result<String> {
    cjson_binding::to_json(value).map_err(|e| Error::UnhandledOwned(format!("to_json: {e}")))
}

/// `cjson_binding::from_json` with the error converted for `?` in tests.
pub(in crate::apps) fn from_json<T: Deserialize + Default>(json: &str) -> Result<T> {
    cjson_binding::from_json(&String::from(json)).map_err(|e| Error::UnhandledOwned(format!("from_json: {e}")))
}
