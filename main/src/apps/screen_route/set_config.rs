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
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result, bytes_to_hex};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::Config;
use crate::apps::display::check::Check;
use crate::apps::display::commons::get_datetime_from_rtc;
use crate::apps::display::input::Input;
use crate::apps::display::date::Date;
use crate::apps::display::select::Select;
use crate::apps::display::time::Time;

use crate::apps::session::User;
use crate::apps::signals::display::DisplayFlag;
use crate::apps::signals::error::ErrorFlag;
use crate::apps::screen_route::auth::{fill_auth_selections, selected_auth_from_selections};
use crate::drivers::date_time::DateTime;
use crate::drivers::platform::Hardware;
use crate::drivers::wifi::Auth;
use crate::traits::hardware::HardwareFn;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute};

static mut FSM_STATE: FSMState = FSMState::Serial;
static mut OLD_FSM_STATE: FSMState = FSMState::Serial;
static UPDATE_DRAW: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Serial,
    Email,
    EmailPasswd,
    EnableWifi,
    Ssid,
    Passwd,
    Auth,
    Date,
    Time,
    EnableDst,
    SetConfig,
    End,
}

pub struct ScreenSetConfig {
    config: &'static mut Config,
    serial: Input,
    email: Input,
    email_passwd: Input,
    wifi_enable: Check,
    wifi_ssid: Input,
    wifi_passwd: Input,
    auth: Select,
    date: Date,
    time: Time,
    enable_dst: Check,
}

impl ScreenRoute for ScreenSetConfig {

     fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits, 
        _status_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
        
    ) -> Result<()> {

        if UPDATE_DRAW.load(Ordering::SeqCst) {
            UPDATE_DRAW.store(false, Ordering::SeqCst);
            *display_signal |= DisplayFlag::Draw as u32;
        }

        let fsm_state = unsafe { *&raw const FSM_STATE };
        
        match fsm_state {
            FSMState::Serial => self.draw_serial_state(lcd, display_signal, rtc)?,
            FSMState::Email => self.draw_email_state(lcd, display_signal, rtc)?,
            FSMState::EmailPasswd => self.draw_email_passwd_state(lcd, display_signal, rtc)?,
            FSMState::EnableWifi => self.draw_enable_wifi_state(lcd, display_signal, rtc)?,
            FSMState::Ssid => self.draw_ssid_state(lcd, display_signal, rtc)?,
            FSMState::Passwd => self.draw_passwd_state(lcd, display_signal, rtc)?,
            FSMState::Auth => self.draw_auth_state(lcd, display_signal, rtc)?,
            FSMState::Date => self.draw_date_state(lcd, display_signal, rtc)?,
            FSMState::Time => self.draw_time_state(lcd, display_signal, rtc)?,
            FSMState::EnableDst => self.draw_enable_dst_state(lcd, display_signal, rtc)?,
            FSMState::SetConfig => self.draw_set_config_state(rtc)?,
            FSMState::End => return Ok(())
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

impl ScreenSetConfig {
    fn draw_serial_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let unique_id = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str(bytes_to_hex(&Hardware::get_unique_id()).as_str());
        let unique_id = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_bytes(&unique_id[..(unique_id.len()/3) * 2]);

        let mut param = ScreenParam::default();
        param.input = Some(unique_id);

        self.serial.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert Serial Number"),
            param,
            Some(|_, confirmed| {
                unsafe { OLD_FSM_STATE = FSM_STATE; }
                if confirmed {
                    unsafe { FSM_STATE = FSMState::Email; }
                }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_email_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let mut param = ScreenParam::<u16>::default();
        param.input = Some(Bytes::from_as_sync_str(self.config.get_session().get_user_local().get_email()));

        self.email.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert Email"),
            param,
            Some(|_, confirmed| {
                unsafe { OLD_FSM_STATE = FSM_STATE; }
                if confirmed {
                    unsafe { FSM_STATE = FSMState::EmailPasswd; }
                } else {
                    unsafe { FSM_STATE = FSMState::Serial; }
                }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_email_passwd_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let mut param = ScreenParam::<u16>::default();
        param.input = Some(Bytes::from_as_sync_str(self.config.get_session().get_user_local().get_password()));
        param.input_secret_mode = Some(true);

        self.email_passwd.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert Password"),
            param,
            Some(|_, confirmed| {
                unsafe { OLD_FSM_STATE = FSM_STATE; }
                if confirmed {
                    unsafe { FSM_STATE = FSMState::EnableWifi; }
                } else {
                    unsafe { FSM_STATE = FSMState::Email; }
                }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_enable_wifi_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let mut param = ScreenParam::default();
        param.check = Some(self.wifi_enable.get_value().unwrap_or(false));

        self.wifi_enable.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Enable WiFi?"),
            param,
            Some(|param, confirmed| {
                if confirmed {
                    unsafe { OLD_FSM_STATE = FSM_STATE; }
                    match param {
                        Some(screen_param) => match screen_param.check {
                            Some(true) => unsafe { FSM_STATE = FSMState::Ssid; },
                            Some(false) => unsafe { FSM_STATE = FSMState::Date; },
                            None => unsafe { FSM_STATE = FSMState::Serial; }
                        },
                        None => unsafe { FSM_STATE = FSMState::Serial; }
                    }
                } else {
                    unsafe { FSM_STATE = FSMState::EmailPasswd; }
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
        param.input = Some(Bytes::from_as_sync_str(self.config.get_wifi_config().get_ssid()));

        self.wifi_ssid.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi SSID"),
            param,
            Some(|_, confirmed| {
                unsafe { OLD_FSM_STATE = FSM_STATE; }
                if confirmed {
                    unsafe { FSM_STATE = FSMState::Passwd; }
                } else {
                    unsafe { FSM_STATE = FSMState::EnableWifi; }
                }
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
        param.input = Some(Bytes::from_as_sync_str(self.config.get_wifi_config().get_password()));

        self.wifi_passwd.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Password"),
            param,
            Some(|_, confirmed| {
                unsafe { OLD_FSM_STATE = FSM_STATE; }
                if confirmed {
                    unsafe { FSM_STATE = FSMState::Auth; }
                } else {
                    unsafe { FSM_STATE = FSMState::Ssid; }
                }
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
        let mut param = ScreenParam::default();
        param.selects = Some(fill_auth_selections(self.config.get_wifi_config().get_auth()));

        self.auth.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Auth"),
            param,
            Some(|_, confirmed| {
                unsafe { OLD_FSM_STATE = FSM_STATE; }
                if confirmed {
                    unsafe { FSM_STATE = FSMState::EnableDst; }
                } else {
                    unsafe { FSM_STATE = FSMState::Passwd; }
                }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_date_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let date_time = get_datetime_from_rtc!(rtc, ErrorFlag::DateTime);

        let mut param = ScreenParam::default();
        param.date_time = Some(date_time.clone());

        self.date.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Set Date"),
            param,
            Some(|_, confirmed| {
                unsafe { OLD_FSM_STATE = FSM_STATE; }
                if confirmed {
                    unsafe { FSM_STATE = FSMState::Time; }
                } else {
                    unsafe { FSM_STATE = FSMState::EnableWifi; }
                }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_time_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let date_time = get_datetime_from_rtc!(rtc, ErrorFlag::DateTime);

        let mut param = ScreenParam::default();
        param.date_time = Some(date_time.clone());

        self.time.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Set Time"),
            param,
            Some(|_, confirmed| {
                unsafe { OLD_FSM_STATE = FSM_STATE; }
                if confirmed {
                    unsafe { FSM_STATE = FSMState::EnableDst; }
                } else {
                    unsafe { FSM_STATE = FSMState::Date; }
                }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_enable_dst_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()> {
        let mut param = ScreenParam::default();
        param.check = Some(self.enable_dst.get_value().unwrap_or(false));

        self.enable_dst.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Enable DST?"),
            param,
            Some(move |_, confirmed| {
                if confirmed {
                    unsafe { FSM_STATE = FSMState::SetConfig; }
                } else {
                    unsafe { FSM_STATE = OLD_FSM_STATE; }
                }
                UPDATE_DRAW.store(true, Ordering::SeqCst);
            }),
        )?;

        Ok(())
    }

    fn draw_set_config_state(&mut self, rtc: &Arc<Mutex<dyn RTC + 'static>>) -> Result<()> {
        let serial = self.serial.get_value()?;
        let email = self.email.get_value()?;
        let email_passwd = self.email_passwd.get_value()?;
        let wifi_enable = self.wifi_enable.get_value()?;

        let mut user = User::default();
        user.set_email(email.as_str());
        user.set_password(email_passwd.as_str());
        self.config.get_session().set_user(&user);

        if wifi_enable {
            let wifi_ssid = self.wifi_ssid.get_value()?;
            let wifi_passwd = self.wifi_passwd.get_value()?;
            let selected_auth = self
                .auth
                .get_value()
                .map(|values| selected_auth_from_selections(&values))
                .unwrap_or(Auth::Open);
            self.config.get_wifi_config().set_ssid(wifi_ssid.as_str());
            self.config.get_wifi_config().set_password(wifi_passwd.as_str());
            self.config.get_wifi_config().set_auth(selected_auth);
            self.config.get_wifi_config().set_enabled(true);
        } else {
            self.config.get_wifi_config().set_ssid("");
            self.config.get_wifi_config().set_password("");
            self.config.get_wifi_config().set_auth(Auth::Open);
            self.config.get_wifi_config().set_enabled(false);
            self.config.get_ntp_config_mut().set_server("");
            self.config.get_ntp_config_mut().set_port(0);
            self.config.get_ntp_config_mut().set_msg_len(0);
            let DateTime{year, month, mday, wday, ..} = self.date.get_value()?;
            let DateTime{hour, minute, second, ..} = self.time.get_value()?;
            let date_time = DateTime::new(year, month, wday, mday, hour, minute, second)?;

            rtc.lock()?.set_timestamp(date_time.to_timestamp())?;
        }

        DateTime::set_daylight_saving_time(self.enable_dst.get_value().unwrap_or_default());

        self.config.set_serial(&Bytes::from_as_sync_str(&serial));

        self.config.apply_locale();
        self.config.apply_daylight_saving_time();
        self.config.apply_ntp();
        self.config.apply_wifi();
        self.config.apply_session();
        Config::save()?;

        unsafe { OLD_FSM_STATE = FSMState::SetConfig; }
        unsafe { FSM_STATE = FSMState::End; }

        Ok(())
    }

    pub const fn new() -> Self {
        Self {
            config: Config::shared(),
            serial: Input::new(),
            email: Input::new(),
            email_passwd: Input::new(),
            wifi_enable: Check::new(),
            wifi_ssid: Input::new(),
            wifi_passwd: Input::new(),
            auth: Select::new(),
            date: Date::new(),
            time: Time::new(),
            enable_dst: Check::new(),
        }
    }
}