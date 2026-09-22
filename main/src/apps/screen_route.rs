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

mod auth;
mod date_time;
mod daylight_saving_time;
mod info;
mod login;
mod menu;
mod set_config;
mod sprinkler;
mod user;
mod wifi;

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::sync::Arc;

use crate::apps::config::Config;
use crate::apps::signals::display::{DisplayFlag, DisplaySignal};
use crate::apps::signals::status::StatusFlag;
use crate::apps::screen_route::date_time::ScreenDateTime;
use crate::apps::screen_route::daylight_saving_time::ScreenDaylightSavingTime;
use crate::apps::screen_route::info::ScreenInfo;
use crate::apps::screen_route::login::ScreenLogin;
use crate::apps::screen_route::set_config::ScreenSetConfig;
use crate::apps::screen_route::menu::ScreenMenu;
use crate::apps::screen_route::sprinkler::ScreenSprinkler;
use crate::apps::screen_route::user::ScreenUser;
use crate::apps::screen_route::wifi::ScreenWifi;
use crate::traits::screen::{Nav, ScreenRoute as ScreenRouteFn, ScreenRouteCtx};
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::signal::Signal;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::Result;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(in crate::apps) enum ScreenId {
    SetConfig,
    Login,
    Menu,
    Info,
    DateTime,
    DaylightSavingTime,
    Wifi,
    User,
    Sprinkler,
}

type BoxedScreen = Box<dyn ScreenRouteFn<ScreenId>>;

/// Marks the current frame as dirty and wakes the display task on the next tick,
/// so a screen that switched its internal state gets drawn right away.
#[inline]
pub(super) fn request_redraw(display_signal: &mut EventBits) {
    *display_signal |= DisplayFlag::Draw as u32;
    DisplaySignal::set(DisplayFlag::Draw as u32);
}

 pub(in crate::apps) struct ScreenRoute {
    config: &'static mut Config,
    stack: Vec<BoxedScreen>,
    check_staus_counter: u8,
}

impl ScreenRoute {
    const CHECK_STATUS_THRESHOLD: u8 = 5;

    const BUTTON_MASK: u32 = DisplayFlag::ButtonPressed as u32
                | DisplayFlag::ButtonReleased as u32
                | DisplayFlag::EncoderButtonPressed as u32
                | DisplayFlag::EncoderButtonReleased as u32;

    pub(super) fn new() -> Self {
        Self {
            config: Config::shared(),
            stack: Vec::new(),
            check_staus_counter: 0,
        }
    }

    fn handle_init(&mut self, status_signal: &mut EventBits) {
        if StatusFlag::CheckConfig.check_signal(*status_signal) {
            self.check_staus_counter += 1;
            if self.check_staus_counter >= Self::CHECK_STATUS_THRESHOLD {
                self.stack.push(Box::new(ScreenSetConfig::new()));
                self.check_staus_counter = 0;
            }
        } else if StatusFlag::Ready.check_signal(*status_signal) {
            self.check_staus_counter += 1;
            if self.check_staus_counter >= Self::CHECK_STATUS_THRESHOLD {
                self.stack.push(Box::new(ScreenMenu::new()));
                self.check_staus_counter = 0;
            }
        } else {
            self.check_staus_counter = 0;
        }
    }

    pub(super) fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn, 
        display_signal: &mut EventBits, 
        status_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>) -> Result<()> {
        
        if self.stack.is_empty() {
            self.handle_init(status_signal);
            if self.stack.is_empty() {
                // Still waiting for the system status: keep the loading screen.
                return Ok(());
            }
            Self::on_screen_changed(display_signal);
        }

        let nav = {
            let Some(top) = self.stack.last_mut() else {
                return Ok(());
            };

            let screen_route_ctx = &mut ScreenRouteCtx {
                lcd,
                display_signal,
                status_signal,
                rtc,
            };

            top.draw(screen_route_ctx)?
        };

        self.navigate(nav, display_signal, status_signal);
        Ok(())
    }

    fn navigate(&mut self, nav: Nav<ScreenId>, display_signal: &mut EventBits, status_signal: &EventBits) {
        match nav {
            Nav::Stay => return,
            Nav::Push(screen) => self.push(screen, status_signal),
            Nav::PushId(id) => self.push(Self::build(id), status_signal),
            Nav::Pop => {
                self.stack.pop();
            }
            Nav::PopTo(id) => {
                while self.stack.last().is_some_and(|screen| screen.id() != id) {
                    self.stack.pop();
                }
            }
            Nav::Replace(screen) => {
                self.stack.pop();
                self.push(screen, status_signal);
            }
        }

        Self::on_screen_changed(display_signal);
    }

    /// Pushes `screen`, or the login screen in its place when it is protected,
    /// a local user exists and nobody is logged in. After the login pops, the
    /// user is back on the screen that was on top (usually the menu).
    fn push(&mut self, screen: BoxedScreen, status_signal: &EventBits) {
        if screen.requires_auth()
            && self.config.get_session().is_set_user_local()
            && !StatusFlag::UserLogged.check_signal(*status_signal)
        {
            self.stack.push(Box::new(ScreenLogin::new()));
        } else {
            self.stack.push(screen);
        }
    }

    fn build(id: ScreenId) -> BoxedScreen {
        match id {
            ScreenId::SetConfig          => Box::new(ScreenSetConfig::new()),
            ScreenId::Login              => Box::new(ScreenLogin::new()),
            ScreenId::Menu               => Box::new(ScreenMenu::new()),
            ScreenId::Info               => Box::new(ScreenInfo::new()),
            ScreenId::DateTime           => Box::new(ScreenDateTime::new()),
            ScreenId::DaylightSavingTime => Box::new(ScreenDaylightSavingTime::new()),
            ScreenId::Wifi               => Box::new(ScreenWifi::new()),
            ScreenId::User               => Box::new(ScreenUser::new()),
            ScreenId::Sprinkler          => Box::new(ScreenSprinkler::new()),
        }
    }

    /// Clears button events so the incoming screen does not see the same (or
    /// bounced) press that triggered the transition, then asks for a redraw.
    #[inline]
    fn on_screen_changed(display_signal: &mut EventBits) {
        *display_signal &= !Self::BUTTON_MASK;
        request_redraw(display_signal);
    }

}
