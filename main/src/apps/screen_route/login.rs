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

use core::any::Any;
use core::sync::atomic::{AtomicBool, Ordering};

use alloc::sync::Arc;
use at_parser_rs::Args;
use at_parser_rs::context::AtContext;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::Config;
use crate::apps::display::input::Input;
use crate::apps::display::text::Text;
use crate::apps::signals::display::DisplayFlag;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute};

static mut FSM_STATE: FSMState = FSMState::Email;
static UPDATE_DRAW: AtomicBool = AtomicBool::new(false);
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

impl ScreenRoute for ScreenLogin {
    #[allow(unused)]
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[allow(unused)]
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits, 
        _status_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> osal_rs::utils::Result<()> {

        if UPDATE_DRAW.load(Ordering::SeqCst) {
            UPDATE_DRAW.store(false, Ordering::SeqCst);
            *display_signal |= DisplayFlag::Draw as u32;
        }

        let fsm_state = unsafe { *&raw const FSM_STATE };

        match fsm_state {
            FSMState::Email => self.handle_email(lcd, display_signal, rtc)?,
            FSMState::EmailPasswd => self.handle_email_passwd(lcd, display_signal, rtc)?,
            FSMState::Status => self.handle_status(lcd, display_signal, rtc)?,
            FSMState::End => return Ok(())
        }

        Err(Error::ReturnWithCode(1))
    }

}

impl ScreenLogin {
    pub(super) fn new() -> Self {
        // Ensure a fresh login flow every time this screen is opened.
        unsafe { FSM_STATE = FSMState::Email; }
        UPDATE_DRAW.store(true, Ordering::SeqCst);

        Self {
            config: Config::shared(),
            email: Input::new(),
            email_passwd: Input::new(),
            status: Text::new(),
        }
    }

    #[inline]
    fn request_draw() {
        UPDATE_DRAW.store(true, Ordering::SeqCst);
    }

    #[inline]
    fn set_state(next: FSMState) {
        unsafe { FSM_STATE = next; }
        Self::request_draw();
    }


    fn handle_email(&mut self,
        lcd: &mut dyn LCDDisplayFn, 
        display_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>
    ) -> Result<()> {
        let mut param = ScreenParam::<u16>::default();
        param.input = Some(Bytes::from_as_sync_str(self.config.get_session().get_user_local().get_email()));

        self.email.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Login Email"),
            param,
            Some(|_, confirmed| {
                if confirmed {
                    Self::set_state(FSMState::EmailPasswd);
                } else {
                    Self::set_state(FSMState::End);
                }
            }),
        )?;

        Ok(())
    }

    fn handle_email_passwd(&mut self, lcd: &mut dyn LCDDisplayFn, display_signal: &mut EventBits, rtc: &Arc<Mutex<dyn RTC + 'static>>) -> Result<()> {
        let mut param = ScreenParam::<u16>::default();
        param.input = Some(Bytes::from_as_sync_str(self.config.get_session().get_user_local().get_password()));
        param.input_secret_mode = Some(true);

        self.email_passwd.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Login Password"),
            param,
            Some(|_, confirmed| {
                if confirmed {
                    Self::set_state(FSMState::Status);
                } else {
                    Self::set_state(FSMState::Email);
                }
            }),
        )?;

        Ok(())
    }

    fn handle_status(&mut self, 
        lcd: &mut dyn LCDDisplayFn, 
        display_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>
    ) -> Result<()> {

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

        self.status.draw(
                    lcd, 
                    display_signal, 
                    rtc, 
                    &text, 
                    ScreenParam::<u16>::default(), 
                    Some(|_, _| {
                        if LOGGED.load(Ordering::SeqCst) {
                            Self::set_state(FSMState::End);
                        } else {
                            // Reset to email state to allow retry
                            Self::set_state(FSMState::Email);
                        }
                    })
                )?;
        Ok(())
    }

    fn handle_end(&mut self, _lcd: &mut dyn LCDDisplayFn, _display_signal: &mut EventBits, _rtc: &Arc<Mutex<dyn RTC + 'static>>) -> Result<()> {
        
        Ok(())
    }
}
