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
use crate::apps::signals::display::DisplayFlag;
use crate::apps::screen_route::ScreenId;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};

static mut FSM_STATE: FSMState = FSMState::Email;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Email,
    Passwd,
    Save,
    End,
}

pub(super) struct ScreenUser {
    email:  Input,
    passwd: Input,
}

impl ScreenRoute<'_, ScreenId> for ScreenUser {
    
    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::User
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {
        match unsafe { *&raw const FSM_STATE } {
            FSMState::Email  => self.draw_email_state(screen_route_ctx),
            FSMState::Passwd => self.draw_passwd_state(screen_route_ctx),
            FSMState::Save   => self.draw_save_state(screen_route_ctx),
            FSMState::End    => Ok(Nav::Pop),
        }
    }
    
}

impl ScreenUser {

    #[inline]
    fn request_draw(display_signal: &mut EventBits) {
        *display_signal |= DisplayFlag::Draw as u32;
    }

    fn draw_email_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {
        let mut param = ScreenParam::<u16>::default();
        param.input = Some(Bytes::from_as_sync_str(
            Config::shared().get_session().get_user_local().get_email(),
        ));

        match self.email.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("User Email"),
            param
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                unsafe { FSM_STATE = FSMState::Passwd; }
                Self::request_draw(display_signal);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => Ok(Nav::Pop),
        }
    }

    fn draw_passwd_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {

        match self.passwd.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("User Password"),
            ScreenParam {
                input: Some(Bytes::from_as_sync_str(
                    Config::shared().get_session().get_user_local().get_password(),
                )),
                input_secret_mode: Some(true),
                ..Default::default()
            }
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                unsafe { FSM_STATE = FSMState::Save; }
                Self::request_draw(display_signal);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => Ok(Nav::Pop),
        }
    }

    fn draw_save_state(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {
        let email  = self.email.get_value()?;
        let passwd = self.passwd.get_value()?;

        let mut user = User::default();
        user.set_email(email.as_str());
        user.set_password(passwd.as_str());
        Config::shared().get_session().set_user(&user);
        Config::shared().apply_session();
        Config::save()?;

        unsafe { FSM_STATE = FSMState::End; }
        Self::request_draw(screen_route_ctx.display_signal);
        Ok(Nav::Stay)
    }

    pub(super) const fn new() -> Self {
        Self {
            email:  Input::new(),
            passwd: Input::new(),
        }
    }
}
