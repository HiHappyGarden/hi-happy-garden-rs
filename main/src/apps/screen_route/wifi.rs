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
use crate::apps::display::check::Check;
use crate::apps::display::input::Input;
use crate::apps::display::select::Select;
use crate::apps::signals::display::DisplayFlag;
use crate::apps::screen_route::auth::{fill_auth_selections, selected_auth_from_selections};
use crate::drivers::wifi::Auth;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute};

static mut FSM_STATE: FSMState = FSMState::Enable;
static UPDATE_DRAW: AtomicBool = AtomicBool::new(false);

#[inline]
fn apply_pending_draw(display_signal: &mut EventBits) {
    if UPDATE_DRAW.load(Ordering::SeqCst) {
        UPDATE_DRAW.store(false, Ordering::SeqCst);
        *display_signal |= DisplayFlag::Draw as u32;
    }
}

#[inline]
fn set_state(next: FSMState) {
    unsafe { FSM_STATE = next; }
    UPDATE_DRAW.store(true, Ordering::SeqCst);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Enable,
    Ssid,
    Passwd,
    AuthType,
    Save,
    End,
}

pub(super) struct ScreenWifi {
    enable:  Check,
    ssid:    Input,
    passwd:  Input,
    auth:    Select,
}

impl ScreenRoute for ScreenWifi {
    fn draw(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        _status_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        apply_pending_draw(display_signal);

        match unsafe { *&raw const FSM_STATE } {
            FSMState::Enable   => self.draw_enable_state(lcd, display_signal, rtc)?,
            FSMState::Ssid     => self.draw_ssid_state(lcd, display_signal, rtc)?,
            FSMState::Passwd   => self.draw_passwd_state(lcd, display_signal, rtc)?,
            FSMState::AuthType => self.draw_auth_state(lcd, display_signal, rtc)?,
            FSMState::Save     => self.draw_save_state()?,
            FSMState::End      => return Ok(())
            
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

impl ScreenWifi {
    fn draw_enable_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let current = Config::shared().get_wifi_config().is_enabled();
        let mut param = ScreenParam::default();
        param.check = Some(self.enable.get_value().unwrap_or(current));

        self.enable.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Enable WiFi?"),
            param,
            Some(|param, confirmed| {
                if confirmed {
                    match param {
                        Some(p) => match p.check {
                            Some(true) => set_state(FSMState::Ssid),
                            _ => set_state(FSMState::Save),
                        },
                        None => set_state(FSMState::End),
                    }
                } else {
                    set_state(FSMState::End);
                }
            }),
        )?;

        Ok(())
    }

    fn draw_ssid_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let mut param = ScreenParam::default();
        param.input = Some(Bytes::from_as_sync_str(&Config::shared().get_wifi_config().get_ssid()));

        self.ssid.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi SSID"),
            param,
            Some(|_, confirmed| {
                set_state(if confirmed { FSMState::Passwd } else { FSMState::Enable });
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
        let mut param = ScreenParam::default();
        param.input = Some(Bytes::from_as_sync_str(&Config::shared().get_wifi_config().get_password()));
        param.input_secret_mode = Some(true);

        self.passwd.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Password"),
            param,
            Some(|_, confirmed| {
                set_state(if confirmed { FSMState::AuthType } else { FSMState::Ssid });
            }),
        )?;

        Ok(())
    }

    fn draw_auth_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let current_auth = Config::shared().get_wifi_config().get_auth();
        let mut param = ScreenParam::default();
        param.selects = Some(fill_auth_selections(current_auth));

        self.auth.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Auth"),
            param,
            Some(|_, confirmed| {
                set_state(if confirmed { FSMState::Save } else { FSMState::Passwd });
            }),
        )?;

        Ok(())
    }

    fn draw_save_state(&mut self) -> Result<()> {
        let wifi_enabled = self.enable.get_value().unwrap_or(false);

        if wifi_enabled {
            let ssid   = self.ssid.get_value()?;
            let passwd = self.passwd.get_value()?;
            let auth_selections = self.auth.get_value()?;
            let selected_auth = selected_auth_from_selections(&auth_selections);

            Config::shared().get_wifi_config().set_ssid(ssid.as_str());
            Config::shared().get_wifi_config().set_password(passwd.as_str());
            Config::shared().get_wifi_config().set_auth(selected_auth);
            Config::shared().get_wifi_config().set_enabled(true);
        } else {
            Config::shared().get_wifi_config().set_ssid("");
            Config::shared().get_wifi_config().set_password("");
            Config::shared().get_wifi_config().set_auth(Auth::Open);
            Config::shared().get_wifi_config().set_enabled(false);
        }

        Config::shared().apply_wifi();
        Config::save()?;

        set_state(FSMState::End);
        Ok(())
    }

    pub(super) const fn new() -> Self {
        Self {
            enable: Check::new(),
            ssid:   Input::new(),
            passwd: Input::new(),
            auth:   Select::new(),
        }
    }
}
