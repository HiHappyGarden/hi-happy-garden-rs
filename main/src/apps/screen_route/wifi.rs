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
use crate::drivers::wifi::Auth;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute, ScreenSelections, screen_selections_new};

static mut FSM_STATE: FSMState = FSMState::Enable;
static UPDATE_DRAW: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Enable,
    Ssid,
    Passwd,
    AuthType,
    Save,
    End,
}

fn auth_as_bytes(auth: Auth) -> Bytes<DISPLAY_INPUT_MAX_SIZE> {
    match auth {
        Auth::Open     => Bytes::from_str("OPEN"),
        Auth::Wpa      => Bytes::from_str("WPA"),
        Auth::Wpa2     => Bytes::from_str("WPA2"),
        Auth::Wpa2Mixed => Bytes::from_str("WPA2 MIXED"),
        Auth::Wpa3     => Bytes::from_str("WPA3"),
        Auth::Wpa2Wpa3 => Bytes::from_str("WPA3/WPA2"),
        _              => Bytes::default(),
    }
}

fn fill_auth_selections(selected: Auth) -> ScreenSelections {
    let mut s = screen_selections_new();
    s[0] = (auth_as_bytes(Auth::Open),     selected == Auth::Open);
    s[1] = (auth_as_bytes(Auth::Wpa),      selected == Auth::Wpa);
    s[2] = (auth_as_bytes(Auth::Wpa2),     selected == Auth::Wpa2);
    s[3] = (auth_as_bytes(Auth::Wpa2Mixed),selected == Auth::Wpa2Mixed);
    s[4] = (auth_as_bytes(Auth::Wpa3),     selected == Auth::Wpa3);
    s[5] = (auth_as_bytes(Auth::Wpa2Wpa3), selected == Auth::Wpa2Wpa3);
    s
}

pub struct ScreenWifi {
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
        if UPDATE_DRAW.load(Ordering::SeqCst) {
            UPDATE_DRAW.store(false, Ordering::SeqCst);
            *display_signal |= DisplayFlag::Draw as u32;
        }

        match unsafe { *&raw const FSM_STATE } {
            FSMState::Enable   => self.draw_enable_state(lcd, display_signal, rtc)?,
            FSMState::Ssid     => self.draw_ssid_state(lcd, display_signal, rtc)?,
            FSMState::Passwd   => self.draw_passwd_state(lcd, display_signal, rtc)?,
            FSMState::AuthType => self.draw_auth_state(lcd, display_signal, rtc)?,
            FSMState::Save     => self.draw_save_state()?,
            FSMState::End      => {
                unsafe { FSM_STATE = FSMState::Enable; }
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
                            Some(true)  => unsafe { FSM_STATE = FSMState::Ssid; },
                            _           => unsafe { FSM_STATE = FSMState::Save; },
                        },
                        None => unsafe { FSM_STATE = FSMState::End; },
                    }
                } else {
                    unsafe { FSM_STATE = FSMState::End; }
                }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
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
        param.input = Some(Bytes::from_as_sync_str(Config::shared().get_wifi_config().get_ssid()));

        self.ssid.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi SSID"),
            param,
            Some(|_, confirmed| {
                unsafe { FSM_STATE = if confirmed { FSMState::Passwd } else { FSMState::Enable }; }
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
        let mut param = ScreenParam::default();
        param.input = Some(Bytes::from_as_sync_str(Config::shared().get_wifi_config().get_password()));
        param.input_secret_mode = Some(true);

        self.passwd.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Password"),
            param,
            Some(|_, confirmed| {
                unsafe { FSM_STATE = if confirmed { FSMState::AuthType } else { FSMState::Ssid }; }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
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
                unsafe { FSM_STATE = if confirmed { FSMState::Save } else { FSMState::Passwd }; }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
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
            let selected_auth = auth_selections
                .iter()
                .find(|v| v.1)
                .map(|v| match v.0.as_str() {
                    "OPEN"       => Auth::Open,
                    "WPA"        => Auth::Wpa,
                    "WPA2"       => Auth::Wpa2,
                    "WPA2 MIXED" => Auth::Wpa2Mixed,
                    "WPA3"       => Auth::Wpa3,
                    "WPA3/WPA2"  => Auth::Wpa2Wpa3,
                    _            => Auth::Open,
                })
                .unwrap_or(Auth::Open);

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

        unsafe { FSM_STATE = FSMState::End; }
        UPDATE_DRAW.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub const fn new() -> Self {
        Self {
            enable: Check::new(),
            ssid:   Input::new(),
            passwd: Input::new(),
            auth:   Select::new(),
        }
    }
}
