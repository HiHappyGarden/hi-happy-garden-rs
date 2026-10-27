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


//! Screen route tests: the navigation stack of [`ScreenRoute`] (start up,
//! push/pop/replace, login in front of protected screens) and the main
//! menu, driven on a [`MockLcd`]. Each screen with logic of its own keeps
//! its tests in a child `tests` module, run from here.

use core::sync::atomic::{AtomicU32, Ordering};

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;

use osal_rs::os::types::EventBits;
use osal_rs::utils::Result;

use super::ScreenRoute as Router;
use super::menu::ScreenMenu;
use crate::apps::config::Config;
use crate::apps::session::User;
use crate::apps::signals::display::DisplayFlag;
use crate::apps::signals::status::StatusFlag;
use crate::apps::test_helpers::{DRAW, MockLcd, mock_rtc, with_draw_restored, with_test_user};
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::screen::{Nav, ScreenRoute as ScreenRouteFn, ScreenRouteCtx};

const TAG: &str = "ScreenRouteTests";

const BUTTONS: EventBits = DisplayFlag::ButtonPressed as EventBits
    | DisplayFlag::ButtonReleased as EventBits
    | DisplayFlag::EncoderButtonPressed as EventBits
    | DisplayFlag::EncoderButtonReleased as EventBits;

const ROTATION: EventBits = DisplayFlag::EncoderRotatedClockwise as EventBits;

/// Screen stub: answers `nav` on each render and counts its `reinit` calls.
struct Stub {
    nav: fn() -> Nav,
    auth: bool,
    reinits: Arc<AtomicU32>,
}

impl Stub {
    const ID: &'static str = "Stub";

    fn new(auth: bool) -> Self {
        Self { nav: || Nav::Stay, auth, reinits: Arc::new(AtomicU32::new(0)) }
    }

    fn with_nav(nav: fn() -> Nav) -> Self {
        Self { nav, ..Self::new(false) }
    }
}

impl ScreenRouteFn for Stub {
    fn id() -> &'static str {
        Self::ID
    }

    fn renderize(&mut self, _: &mut ScreenRouteCtx<'_>) -> Result<Nav> {
        Ok((self.nav)())
    }

    fn requires_auth(&self) -> bool {
        self.auth
    }

    fn reinit(&mut self) {
        self.reinits.fetch_add(1, Ordering::Relaxed);
    }
}

fn ids(router: &Router) -> alloc::vec::Vec<&'static str> {
    router.stack.iter().map(|(id, _)| *id).collect()
}

/// Runs `test` with no local user, restoring the real one afterwards.
fn without_local_user<R>(test: impl FnOnce() -> Result<R>) -> Result<R> {
    let session = Config::shared().get_session();
    let backup = session.get_user_local();
    session.set_user(&User::default());
    let ret = test();
    Config::shared().get_session().set_user(&backup);
    ret
}

// ---------------------------------------------------------------- start up

fn test_init_waits_status() -> Result<()> {
    let mut router = Router::new();
    let mut status: EventBits = 0;
    for _ in 0..2 * Router::CHECK_STATUS_THRESHOLD {
        router.handle_init(&mut status);
    }
    test_assert!(router.stack.is_empty(), "screen pushed without a system status");
    Ok(())
}

fn test_init_ready_opens_menu() -> Result<()> {
    let mut router = Router::new();
    let mut status: EventBits = StatusFlag::Ready.into();
    for _ in 1..Router::CHECK_STATUS_THRESHOLD {
        router.handle_init(&mut status);
    }
    test_assert!(router.stack.is_empty(), "menu opened before the threshold");
    router.handle_init(&mut status);
    test_assert!(ids(&router) == [ScreenMenu::id()], "{:?}", ids(&router));
    test_assert_eq!(router.check_staus_counter, 0);
    Ok(())
}

fn test_init_check_config_opens_wizard() -> Result<()> {
    // The wizard wins over the menu when both flags are up
    for status in [StatusFlag::CheckConfig as EventBits, StatusFlag::CheckConfig as EventBits | StatusFlag::Ready as EventBits] {
        let mut router = Router::new();
        let mut status = status;
        for _ in 0..Router::CHECK_STATUS_THRESHOLD {
            router.handle_init(&mut status);
        }
        test_assert!(ids(&router) == ["ScreenWizard"], "status {status:#x}: {:?}", ids(&router));
    }
    Ok(())
}

fn test_init_counter_reset() -> Result<()> {
    let mut router = Router::new();
    let mut ready: EventBits = StatusFlag::Ready.into();
    let mut idle: EventBits = 0;
    for _ in 1..Router::CHECK_STATUS_THRESHOLD {
        router.handle_init(&mut ready);
    }
    router.handle_init(&mut idle);
    for _ in 1..Router::CHECK_STATUS_THRESHOLD {
        router.handle_init(&mut ready);
    }
    test_assert!(router.stack.is_empty(), "the count survived a status drop");
    router.handle_init(&mut ready);
    test_assert!(ids(&router) == [ScreenMenu::id()], "{:?}", ids(&router));
    Ok(())
}

// -------------------------------------------------------------- navigation

fn test_navigate_stay() -> Result<()> {
    let mut router = Router::new();
    router.stack.push((Stub::ID, Box::new(Stub::new(false))));
    let mut signal = BUTTONS;
    router.navigate(Nav::Stay, &mut signal, &0);
    test_assert_eq!(router.stack.len(), 1);
    test_assert_eq!(signal, BUTTONS, "stay touched the display signal");
    Ok(())
}

fn test_navigate_push() -> Result<()> {
    with_draw_restored(|| {
        let mut router = Router::new();
        let mut signal = BUTTONS | ROTATION;
        router.navigate(Nav::Push { id: Stub::ID, screen: Box::new(Stub::new(false)) }, &mut signal, &0);
        test_assert!(ids(&router) == [Stub::ID], "{:?}", ids(&router));
        test_assert_eq!(signal & BUTTONS, 0, "button events leaked to the new screen");
        test_assert!(signal & ROTATION != 0, "rotation cleared");
        test_assert!(signal & DRAW != 0, "no redraw after push");
        Ok(())
    })
}

fn test_navigate_pop() -> Result<()> {
    with_draw_restored(|| {
        let mut router = Router::new();
        let parent = Stub::new(false);
        let reinits = Arc::clone(&parent.reinits);
        router.stack.push(("Parent", Box::new(parent)));

        router.stack.push((Stub::ID, Box::new(Stub::new(false))));
        let mut signal = BUTTONS;
        router.navigate(Nav::Pop(false), &mut signal, &0);
        test_assert!(ids(&router) == ["Parent"], "{:?}", ids(&router));
        test_assert_eq!(reinits.load(Ordering::Relaxed), 0, "parent reinit without request");
        test_assert_eq!(signal & BUTTONS, 0);
        test_assert!(signal & DRAW != 0, "no redraw after pop");

        router.stack.push((Stub::ID, Box::new(Stub::new(false))));
        router.navigate(Nav::Pop(true), &mut signal, &0);
        test_assert!(ids(&router) == ["Parent"], "{:?}", ids(&router));
        test_assert_eq!(reinits.load(Ordering::Relaxed), 1, "parent not reinit");
        Ok(())
    })
}

fn test_navigate_pop_empty() -> Result<()> {
    let mut router = Router::new();
    let mut signal = BUTTONS;
    router.navigate(Nav::Pop(true), &mut signal, &0);
    test_assert!(router.stack.is_empty());
    test_assert_eq!(signal, BUTTONS, "pop on an empty stack touched the display signal");
    Ok(())
}

fn test_navigate_replace() -> Result<()> {
    with_draw_restored(|| {
        let mut router = Router::new();
        router.stack.push(("Bottom", Box::new(Stub::new(false))));
        router.stack.push(("Top", Box::new(Stub::new(false))));
        let mut signal: EventBits = 0;
        router.navigate(Nav::Replace { id: "New", screen: Box::new(Stub::new(false)) }, &mut signal, &0);
        test_assert!(ids(&router) == ["Bottom", "New"], "{:?}", ids(&router));
        test_assert!(signal & DRAW != 0, "no redraw after replace");
        Ok(())
    })
}

fn test_push_requires_login() -> Result<()> {
    with_test_user(|| {
        let mut router = Router::new();
        router.push(Stub::ID, Box::new(Stub::new(true)), &0);
        test_assert!(ids(&router) == ["ScreenLogin"], "protected screen opened without login: {:?}", ids(&router));

        let mut router = Router::new();
        router.push(Stub::ID, Box::new(Stub::new(true)), &StatusFlag::UserLogged.into());
        test_assert!(ids(&router) == [Stub::ID], "logged user sent to login: {:?}", ids(&router));

        let mut router = Router::new();
        router.push(Stub::ID, Box::new(Stub::new(false)), &0);
        test_assert!(ids(&router) == [Stub::ID], "public screen behind login: {:?}", ids(&router));
        Ok(())
    })
}

fn test_push_without_local_user() -> Result<()> {
    without_local_user(|| {
        let mut router = Router::new();
        router.push(Stub::ID, Box::new(Stub::new(true)), &0);
        test_assert!(ids(&router) == [Stub::ID], "login asked with no local user: {:?}", ids(&router));
        Ok(())
    })
}

// -------------------------------------------------------------------- draw

fn test_draw_empty_stack() -> Result<()> {
    let mut router = Router::new();
    let mut lcd = MockLcd::new();
    let rtc = mock_rtc(0);
    let (mut display, mut status): (EventBits, EventBits) = (0, 0);
    router.draw(&mut lcd, &mut display, &mut status, &rtc)?;
    test_assert!(router.stack.is_empty());
    test_assert_eq!(display, 0, "loading screen redrawn");
    test_assert!(lcd.texts.is_empty() && lcd.rects.is_empty(), "something drawn over the loading screen");
    Ok(())
}

fn test_draw_applies_nav() -> Result<()> {
    with_draw_restored(|| {
        let mut router = Router::new();
        let parent = Stub::new(false);
        let reinits = Arc::clone(&parent.reinits);
        router.stack.push(("Parent", Box::new(parent)));
        router.stack.push((Stub::ID, Box::new(Stub::with_nav(|| Nav::Pop(true)))));

        let mut lcd = MockLcd::new();
        let rtc = mock_rtc(0);
        let (mut display, mut status): (EventBits, EventBits) = (0, 0);
        router.draw(&mut lcd, &mut display, &mut status, &rtc)?;
        test_assert!(ids(&router) == ["Parent"], "{:?}", ids(&router));
        test_assert_eq!(reinits.load(Ordering::Relaxed), 1);
        test_assert!(display & DRAW != 0, "no redraw after the top screen popped");
        Ok(())
    })
}

// -------------------------------------------------------------------- menu

/// Renders `menu` once with `flags` and returns its answer and the label drawn.
fn menu_frame(menu: &mut ScreenMenu, flags: EventBits) -> Result<(Nav, String)> {
    let mut lcd = MockLcd::new();
    let rtc = mock_rtc(0);
    let (mut display, mut status) = (flags, 0);
    let nav = menu.renderize(&mut ScreenRouteCtx {
        lcd: &mut lcd,
        display_signal: &mut display,
        status_signal: &mut status,
        rtc: &rtc,
    })?;
    let label = lcd.texts.last().map(|t| String::from_utf8_lossy(&t.text).into_owned()).unwrap_or_default();
    Ok((nav, label))
}

fn test_menu_rotation() -> Result<()> {
    const LABELS: [&str; 7] = ["Info", "Date Time", "Daylight Saving Time", "Wifi", "User", "Sprinkler", "Manual"];
    let mut menu = ScreenMenu::new();
    let (nav, label) = menu_frame(&mut menu, 0)?;
    test_assert!(matches!(nav, Nav::Stay));
    test_assert_eq!(label, LABELS[0]);

    // A full turn clockwise, back on the first entry
    for expected in LABELS.iter().cycle().skip(1).take(LABELS.len()) {
        let (_, label) = menu_frame(&mut menu, DisplayFlag::EncoderRotatedClockwise as EventBits)?;
        // A label wider than the display scrolls: any window of it will do
        let mut ring = String::from(*expected);
        ring.push_str("  ");
        ring.push_str(expected);
        test_assert!(ring.contains(label.as_str()), "{label:?} is not {expected:?}");
    }

    let (_, label) = menu_frame(&mut menu, DisplayFlag::EncoderRotatedCounterClockwise as EventBits)?;
    test_assert_eq!(label, "Manual", "counter-clockwise from the first entry");
    Ok(())
}

fn test_menu_confirm_pushes_item() -> Result<()> {
    let mut menu = ScreenMenu::new();
    menu_frame(&mut menu, DisplayFlag::EncoderRotatedClockwise as EventBits)?;
    let (nav, _) = menu_frame(&mut menu, DisplayFlag::EncoderButtonReleased as EventBits)?;
    test_assert!(matches!(nav, Nav::Push { id: "Date Time", .. }), "confirm did not open the entry");

    let (nav, _) = menu_frame(&mut menu, DisplayFlag::ButtonReleased as EventBits)?;
    test_assert!(matches!(nav, Nav::Stay), "back left the main menu");
    Ok(())
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_init_waits_status,
        test_init_ready_opens_menu,
        test_init_check_config_opens_wizard,
        test_init_counter_reset,
        test_navigate_stay,
        test_navigate_push,
        test_navigate_pop,
        test_navigate_pop_empty,
        test_navigate_replace,
        test_push_requires_login,
        test_push_without_local_user,
        test_draw_empty_stack,
        test_draw_applies_nav,
        test_menu_rotation,
        test_menu_confirm_pushes_item,
    );
    super::zone_detail::tests::run_all_tests(stats);
}
