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
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::Config;
use crate::apps::display::input::Input;
use crate::apps::session::User;
use crate::apps::signals::display::DisplayFlag;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute};

static mut FSM_STATE: FSMState = FSMState::Email;
static UPDATE_DRAW: AtomicBool = AtomicBool::new(false);

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

impl ScreenRoute for ScreenUser {
    fn draw(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        _status_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        if UPDATE_DRAW.load(Ordering::SeqCst) {
            UPDATE_DRAW.store(false, Ordering::SeqCst);
            *display_signal |= DisplayFlag::Draw as u32;
        }

        match unsafe { *&raw const FSM_STATE } {
            FSMState::Email  => self.draw_email_state(lcd, display_signal, rtc)?,
            FSMState::Passwd => self.draw_passwd_state(lcd, display_signal, rtc)?,
            FSMState::Save   => self.draw_save_state()?,
            FSMState::End    => {
                unsafe { FSM_STATE = FSMState::Email; }
                return Ok(());
            }
        }

        Err(Error::ReturnWithCode(1))
    }

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
}

impl ScreenUser {
    fn draw_email_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let mut param = ScreenParam::<u16>::default();
        param.input = Some(Bytes::from_as_sync_str(
            Config::shared().get_session().get_user_local().get_email(),
        ));

        self.email.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("User Email"),
            param,
            Some(|_, confirmed| {
                unsafe { FSM_STATE = if confirmed { FSMState::Passwd } else { FSMState::End }; }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_passwd_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let mut param = ScreenParam::<u16>::default();
        param.input = Some(Bytes::from_as_sync_str(
            Config::shared().get_session().get_user_local().get_password(),
        ));
        param.input_secret_mode = Some(true);

        self.passwd.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("User Password"),
            param,
            Some(|_, confirmed| {
                unsafe { FSM_STATE = if confirmed { FSMState::Save } else { FSMState::Email }; }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_save_state(&mut self) -> Result<()> {
        let email  = self.email.get_value()?;
        let passwd = self.passwd.get_value()?;

        let mut user = User::default();
        user.set_email(email.as_str());
        user.set_password(passwd.as_str());
        Config::shared().get_session().set_user(&user);
        Config::shared().apply_session();
        Config::save()?;

        unsafe { FSM_STATE = FSMState::End; }
        UPDATE_DRAW.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub(super) const fn new() -> Self {
        Self {
            email:  Input::new(),
            passwd: Input::new(),
        }
    }
}
