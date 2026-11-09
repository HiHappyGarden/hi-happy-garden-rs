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

use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::display::input::Input;
use crate::apps::session::User;
use crate::apps::signals::display::request_redraw;
use crate::apps::config::config_shared;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Email,
    Passwd,
}

pub(super) struct ScreenUser {
    fsm_state: FSMState,
    email:  Input,
    passwd: Input,
}

impl ScreenRoute for ScreenUser {
    fn id() -> &'static str {
        "ScreenUser"
    }

    fn renderize(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav> {
        match self.fsm_state {
            FSMState::Email  => self.draw_email_state(screen_route_ctx),
            FSMState::Passwd => self.draw_passwd_state(screen_route_ctx),
        }
    }
    
}

impl ScreenUser {

    #[inline]
    fn set_state(&mut self, display_signal: &mut EventBits, next: FSMState) {
        self.fsm_state = next;
        request_redraw(display_signal);
    }

    fn draw_email_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav> {
       
        match self.email.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("User Email"),
            ScreenParam::Input { value: config_shared!(Error::NullPtr).get_session().get_user_local().get_email().clone(), secret_mode: false }
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.set_state(display_signal, FSMState::Passwd);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => Ok(Nav::Pop(false)),
        }
    }

    fn draw_passwd_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav> {

        // The stored password is a SHA256 hash, so there is nothing to prefill.
        match self.passwd.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("User Password"),
            ScreenParam::Input { value: Bytes::default(), secret_mode: true }
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.save()?;
                Ok(Nav::Pop(false))
            }
            Answer::Cancelled => {
                self.set_state(display_signal, FSMState::Email);
                Ok(Nav::Stay)
            }
        }
    }

    fn save(&mut self) -> Result<()> {
        let email  = self.email.get_value()?;
        let passwd = self.passwd.get_value()?;

        let user = User::from_credentials(email.as_str(), passwd.as_str())?;

        let mut config = config_shared!(Error::NullPtr);
        config.get_session().set_user(&user);
        config.apply_session();
        config.save()?;

        Ok(())
    }

    pub(super) const fn new() -> Self {
        Self {
            fsm_state: FSMState::Email,
            email:  Input::new(),
            passwd: Input::new(),
        }
    }
}
