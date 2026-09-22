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

use core::sync::atomic::{AtomicBool, Ordering};

use at_parser_rs::Args;
use at_parser_rs::context::AtContext;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::Config;
use crate::apps::display::input::Input;
use crate::apps::display::text::Text;
use crate::apps::screen_route::ScreenId;
use crate::apps::signals::display::DisplayFlag;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};

static mut FSM_STATE: FSMState = FSMState::Email;
static LOGGED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Email,
    EmailPasswd,
    Status,
    End,
}

pub(super) struct ScreenLogin {
    config: &'static mut Config,
    email: Input,
    email_passwd: Input,
    status: Text,
}

impl ScreenRoute<'_, ScreenId> for ScreenLogin {


    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::Login
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {

        match unsafe { *&raw const FSM_STATE } {
            FSMState::Email => self.handle_email(screen_route_ctx),
            FSMState::EmailPasswd => self.handle_email_passwd(screen_route_ctx),
            FSMState::Status => self.handle_status(screen_route_ctx),
            FSMState::End => Ok(Nav::Pop),
        }
    }

}

impl ScreenLogin {
    pub(super) fn new() -> Self {
        // Ensure a fresh login flow every time this screen is opened.
        unsafe { FSM_STATE = FSMState::Email; }

        Self {
            config: Config::shared(),
            email: Input::new(),
            email_passwd: Input::new(),
            status: Text::new(),
        }
    }

    #[inline]
    fn request_draw(display_signal: &mut EventBits) {
        *display_signal |= DisplayFlag::Draw as u32;
    }

    #[inline]
    fn set_state(display_signal: &mut EventBits, next: FSMState) {
        unsafe { FSM_STATE = next; }
        Self::request_draw(display_signal);
    }


    fn handle_email(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {

        match self.email.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Login Email"),
            ScreenParam::<u16>::default(),
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                Self::set_state(display_signal, FSMState::EmailPasswd);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => {
                Self::set_state(display_signal, FSMState::End);
                Ok(Nav::Stay)
            }
        }
    }

    fn handle_email_passwd(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {
        
        match self.email_passwd.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Login Password"),
            ScreenParam::<u16>::default(),

        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                Self::set_state(display_signal, FSMState::Status);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => {
                Self::set_state(display_signal, FSMState::Email);
                Ok(Nav::Stay)
            }
        }   
    }

    fn handle_status(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {

        let email = self.email.get_value()?;
        let email_passwd = self.email_passwd.get_value()?;

        let mut cmd = Bytes::<DISPLAY_INPUT_MAX_SIZE>::new();
        cmd.format(format_args!("AT+SESS=i,{email},{email_passwd}", email = email.as_str(), email_passwd = email_passwd.as_str()));

        let args: Args = Args {
            raw: cmd.as_str()
        };

        match self.config.get_session().set("", args) {
            Ok(_) => {
                if let Ok(_) = self.config.get_session().exec("") {
                    LOGGED.store(true, Ordering::SeqCst);
                } else {
                    LOGGED.store(false, Ordering::SeqCst);
                }
            },
            Err(_) => {
                LOGGED.store(false, Ordering::SeqCst);
            }
        };


        let mut text = Bytes::<DISPLAY_INPUT_MAX_SIZE>::new();
        if LOGGED.load(Ordering::SeqCst) {
            text.append_str("Login successful");
        } else {
            text.append_str("Login failed");
        }

        match self.status.draw(
                    *lcd, 
                    display_signal, 
                    rtc, 
                    &text, 
                    ScreenParam::<u16>::default()
                )?  
                {
                Answer::Pending => Ok(Nav::Stay),
                Answer::Confirmed(_) => {
                    if LOGGED.load(Ordering::SeqCst) {
                        Self::set_state(display_signal, FSMState::End);
                    } else {
                        Self::set_state(display_signal, FSMState::Email);
                    }
                    Ok(Nav::Stay)
                }
                Answer::Cancelled => {
                    Self::set_state(display_signal, FSMState::Email);
                    Ok(Nav::Stay)
                }
        }
        
    }

}
