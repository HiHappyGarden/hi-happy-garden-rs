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

use alloc::string::String;

use at_parser_rs::Args;
use at_parser_rs::context::AtContext;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::Config;
use crate::apps::display::input::Input;
use crate::apps::display::text::Text;
use crate::apps::screen_route::{ScreenId, request_redraw};
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Email,
    EmailPasswd,
    Status,
}

pub(super) struct ScreenLogin {
    config: &'static mut Config,
    fsm_state: FSMState,
    logged: bool,
    email: Input,
    email_passwd: Input,
    status: Text,
}

impl ScreenRoute<ScreenId> for ScreenLogin {

    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::Login
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        match self.fsm_state {
            FSMState::Email => self.handle_email(screen_route_ctx),
            FSMState::EmailPasswd => self.handle_email_passwd(screen_route_ctx),
            FSMState::Status => self.handle_status(screen_route_ctx),
        }
    }

    fn requires_auth(&self) -> bool {
        false
    }
}

impl ScreenLogin {
    pub(super) fn new() -> Self {
        Self {
            config: Config::shared(),
            fsm_state: FSMState::Email,
            logged: false,
            email: Input::new(),
            email_passwd: Input::new(),
            status: Text::new(),
        }
    }

    #[inline]
    fn set_state(&mut self, display_signal: &mut EventBits, next: FSMState) {
        self.fsm_state = next;
        request_redraw(display_signal);
    }

    fn handle_email(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {

        match self.email.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Login Email"),
            ScreenParam::<u16>::default(),
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.set_state(display_signal, FSMState::EmailPasswd);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => Ok(Nav::Pop),
        }
    }

    fn handle_email_passwd(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        
        match self.email_passwd.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Login Password"),
            ScreenParam::<u16>::default(),
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.logged = self.login()?;
                self.set_state(display_signal, FSMState::Status);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => {
                self.set_state(display_signal, FSMState::Email);
                Ok(Nav::Stay)
            }
        }   
    }

    fn handle_status(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {

        let text = if self.logged {
            Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Login successful")
        } else {
            Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Login failed")
        };

        match self.status.draw(
            *lcd, 
            display_signal, 
            rtc, 
            &text, 
            ScreenParam::<u16>::default()
        )? {
            Answer::Pending => Ok(Nav::Stay),
            // Any button: leave on success, retry from the email otherwise.
            Answer::Confirmed(_) | Answer::Cancelled => {
                if self.logged {
                    Ok(Nav::Pop)
                } else {
                    self.set_state(display_signal, FSMState::Email);
                    Ok(Nav::Stay)
                }
            }
        }
    }

    fn login(&mut self) -> Result<bool> {
        let email = self.email.get_value()?;
        let email_passwd = self.email_passwd.get_value()?;

        // Only the arguments, as the AT parser would pass them: the command
        // prefix would end up in the first argument.
        let mut raw = String::from("i,");
        Self::push_quoted(&mut raw, email.as_str());
        raw.push(',');
        Self::push_quoted(&mut raw, email_passwd.as_str());

        let args: Args = Args {
            raw: raw.as_str()
        };

        let session = self.config.get_session();
        Ok(session.set("", args).is_ok() && session.exec("").is_ok())
    }

    /// Quotes `value` so that commas and quotes typed by the user do not split the arguments.
    fn push_quoted(raw: &mut String, value: &str) {
        raw.push('"');
        for ch in value.chars() {
            if ch == '"' || ch == '\\' {
                raw.push('\\');
            }
            raw.push(ch);
        }
        raw.push('"');
    }

}
