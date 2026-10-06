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
mod wizard;
mod schedule;
mod sprinkler;
mod user;
mod wifi;
mod zone;

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::sync::Arc;

use crate::apps::config::Config;
use crate::apps::signals::display::{DisplayFlag, request_redraw};
use crate::apps::signals::status::StatusFlag;
use crate::apps::screen_route::login::ScreenLogin;
use crate::apps::screen_route::wizard::ScreenWizard;
use crate::apps::screen_route::menu::ScreenMenu;
use crate::traits::screen::{BoxedScreenRoute, Nav, ScreenRoute as ScreenRouteFn, ScreenRouteCtx};
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::Result;

 pub(in crate::apps) struct ScreenRoute {
    config: &'static mut Config,
    stack: Vec<(&'static str, BoxedScreenRoute)>,
    check_staus_counter: u8,
}

impl ScreenRoute {
    const CHECK_STATUS_THRESHOLD: u8 = 5;

    const BUTTON_MASK: EventBits = DisplayFlag::ButtonPressed as EventBits
                | DisplayFlag::ButtonReleased as EventBits
                | DisplayFlag::EncoderButtonPressed as EventBits
                | DisplayFlag::EncoderButtonReleased as EventBits;

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
                self.stack.push((ScreenWizard::id(), Box::new(ScreenWizard::new())));
                self.check_staus_counter = 0;
            }
        } else if StatusFlag::Ready.check_signal(*status_signal) {
            self.check_staus_counter += 1;
            if self.check_staus_counter >= Self::CHECK_STATUS_THRESHOLD {
                self.stack.push((ScreenMenu::id(), Box::new(ScreenMenu::new())));
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

            top.1.renderize(screen_route_ctx)?
        };

        self.navigate(nav, display_signal, status_signal);
        Ok(())
    }

    fn navigate(&mut self, nav: Nav, display_signal: &mut EventBits, status_signal: &EventBits) {
        match nav {
            Nav::Stay => return,
            Nav::Push{ id, screen } => self.push(id, screen, status_signal),
            Nav::Pop => {
                self.stack.pop();
            }
            Nav::Replace{ id, screen } => {
                self.stack.pop();
                self.stack.push((id, screen));
            }
        }

        Self::on_screen_changed(display_signal);
    }

    /// Pushes `screen`, or the login screen in its place when it is protected,
    /// a local user exists and nobody is logged in. After the login pops, the
    /// user is back on the screen that was on top (usually the menu).
    fn push(&mut self, id: &'static str, screen: BoxedScreenRoute, status_signal: &EventBits) {

        let user = self.config.get_session().get_user_local();


        if screen.requires_auth()
            && user.is_empty_passwd()
            && self.config.get_session().is_set_user_local()
            && !StatusFlag::UserLogged.check_signal(*status_signal)
        {
            self.stack.push((ScreenLogin::id(), Box::new(ScreenLogin::new())));
        } else {
            self.stack.push((id, screen));
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
