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
use osal_rs::utils::{Bytes, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::Config;
use crate::apps::display::input::Input;
use crate::apps::session::User;
use crate::apps::screen_route::ScreenId;
use crate::apps::signals::display::request_redraw;
use crate::drivers::encrypt::EncryptGeneric;
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

impl ScreenRoute<ScreenId> for ScreenUser {
    
    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::User
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
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

    fn draw_email_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
       
        match self.email.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("User Email"),
            ScreenParam::Input { value: Config::shared().get_session().get_user_local().get_email().clone(), secret_mode: false }
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.set_state(display_signal, FSMState::Passwd);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => Ok(Nav::Pop),
        }
    }

    fn draw_passwd_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {

        // The stored password is a SHA256 hash, so there is nothing to prefill.
        match self.passwd.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("User Password"),
            ScreenParam::Input { value: Bytes::default(), secret_mode: true }
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.save()?;
                Ok(Nav::Pop)
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

        let mut user = User::default();
        user.set_email(email.as_str());
        user.set_password(EncryptGeneric::get_sha256(passwd.to_bytes())?.as_str());
        Config::shared().get_session().set_user(&user);
        Config::shared().apply_session();
        Config::save()?;
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
