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

use alloc::boxed::Box;
use alloc::sync::Arc;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Result, bytes_to_hex};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::Config;
use crate::apps::display::check::Check;
use crate::apps::display::commons::get_datetime_from_rtc;
use crate::apps::display::date::Date;
use crate::apps::display::input::Input;
use crate::apps::display::select::Select;
use crate::apps::display::time::Time;
use crate::apps::session::User;
use crate::apps::signals::error::ErrorFlag;
use crate::apps::screen_route::auth::{fill_auth_selections, selected_auth_from_selections};
use crate::apps::screen_route::menu::ScreenMenu;
use crate::apps::screen_route::{ScreenId, request_redraw};
use crate::drivers::date_time::DateTime;
use crate::drivers::encrypt::EncryptGeneric;
use crate::drivers::platform::Hardware;
use crate::drivers::wifi::Auth;
use crate::traits::hardware::HardwareFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};

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
}

pub(super) struct ScreenSetConfig {
    config: &'static mut Config,
    fsm_state: FSMState,
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

impl ScreenRoute<ScreenId> for ScreenSetConfig {
    
    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::SetConfig
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        match self.fsm_state {
            FSMState::Serial      => self.draw_serial_state(screen_route_ctx),
            FSMState::Email       => self.draw_email_state(screen_route_ctx),
            FSMState::EmailPasswd => self.draw_email_passwd_state(screen_route_ctx),
            FSMState::EnableWifi  => self.draw_enable_wifi_state(screen_route_ctx),
            FSMState::Ssid        => self.draw_ssid_state(screen_route_ctx),
            FSMState::Passwd      => self.draw_passwd_state(screen_route_ctx),
            FSMState::Auth        => self.draw_auth_state(screen_route_ctx),
            FSMState::Date        => self.draw_date_state(screen_route_ctx),
            FSMState::Time        => self.draw_time_state(screen_route_ctx),
            FSMState::EnableDst   => self.draw_enable_dst_state(screen_route_ctx),
        }
    }

    fn requires_auth(&self) -> bool {
        false
    }
}

impl ScreenSetConfig {

    #[inline]
    fn set_state(&mut self, display_signal: &mut EventBits, next: FSMState) {
        self.fsm_state = next;
        request_redraw(display_signal);
    }

    /// Moves to `confirmed` or `cancelled` according to the widget answer.
    fn step<N, const N_SELECTS: usize>(&mut self, display_signal: &mut EventBits, answer: Answer<N, N_SELECTS>, confirmed: FSMState, cancelled: FSMState) -> Nav<ScreenId>
    where N: crate::traits::integer::Integer
    {
        match answer {
            Answer::Pending => {}
            Answer::Confirmed(_) => self.set_state(display_signal, confirmed),
            Answer::Cancelled => self.set_state(display_signal, cancelled),
        }
        Nav::Stay
    }

    fn draw_serial_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let unique_id = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str(bytes_to_hex(&Hardware::get_unique_id()).as_str());
        let unique_id = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_bytes(&unique_id[..(unique_id.len()/3) * 2]);

        let mut param = ScreenParam::default();
        param.input = Some(unique_id);

        let answer = self.serial.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert Serial Number"),
            param,
        )?;

        // First step: there is nothing to go back to.
        Ok(self.step(display_signal, answer, FSMState::Email, FSMState::Serial))
    }

    fn draw_email_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::<u16>::default();
        param.input = Some(Bytes::from_as_sync_str(self.config.get_session().get_user_local().get_email()));

        let answer = self.email.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert Email"),
            param,
        )?;

        Ok(self.step(display_signal, answer, FSMState::EmailPasswd, FSMState::Serial))
    }

    fn draw_email_passwd_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        // The stored password is a SHA256 hash, so there is nothing to prefill.
        let mut param = ScreenParam::<u16>::default();
        param.input_secret_mode = Some(true);

        let answer = self.email_passwd.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert Password"),
            param,
        )?;

        Ok(self.step(display_signal, answer, FSMState::EnableWifi, FSMState::Email))
    }

    fn draw_enable_wifi_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::default();
        param.check = Some(false);

        match self.wifi_enable.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Enable WiFi?"),
            param,
        )? {
            Answer::Pending => {}
            Answer::Confirmed(param) => {
                // With WiFi the clock comes from NTP, otherwise it is set by hand.
                let next = if param.check.unwrap_or(false) { FSMState::Ssid } else { FSMState::Date };
                self.set_state(display_signal, next);
            }
            Answer::Cancelled => self.set_state(display_signal, FSMState::EmailPasswd),
        }

        Ok(Nav::Stay)
    }

    fn draw_ssid_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::default();
        param.input = Some(Bytes::from_as_sync_str(&self.config.get_wifi_config().get_ssid()));

        let answer = self.wifi_ssid.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi SSID"),
            param,
        )?;

        Ok(self.step(display_signal, answer, FSMState::Passwd, FSMState::EnableWifi))
    }

    fn draw_passwd_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::default();
        param.input = Some(Bytes::from_as_sync_str(&self.config.get_wifi_config().get_password()));

        let answer = self.wifi_passwd.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Password"),
            param,
        )?;

        Ok(self.step(display_signal, answer, FSMState::Auth, FSMState::Ssid))
    }

    fn draw_auth_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::default();
        param.selects = Some(fill_auth_selections(self.config.get_wifi_config().get_auth()));

        let answer = self.auth.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Auth"),
            param,
        )?;

        Ok(self.step(display_signal, answer, FSMState::EnableDst, FSMState::Passwd))
    }

    fn draw_date_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::default();
        param.date_time = Some(get_datetime_from_rtc!(rtc, ErrorFlag::DateTime));

        let answer = self.date.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Set Date"),
            param,
        )?;

        Ok(self.step(display_signal, answer, FSMState::Time, FSMState::EnableWifi))
    }

    fn draw_time_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::default();
        param.date_time = Some(get_datetime_from_rtc!(rtc, ErrorFlag::DateTime));

        let answer = self.time.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Set Time"),
            param,
        )?;

        Ok(self.step(display_signal, answer, FSMState::EnableDst, FSMState::Date))
    }

    fn draw_enable_dst_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        let mut param = ScreenParam::default();
        param.check = Some(false);

        match self.enable_dst.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Enable DST?"),
            param,
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.save(rtc)?;
                // The wizard is the root of the stack: the menu takes its place.
                Ok(Nav::Replace(Box::new(ScreenMenu::new())))
            }
            Answer::Cancelled => {
                let previous = if self.wifi_enable.get_value().unwrap_or(false) { FSMState::Auth } else { FSMState::Time };
                self.set_state(display_signal, previous);
                Ok(Nav::Stay)
            }
        }
    }

    fn save(&mut self, rtc: &Arc<Mutex<dyn RTC + 'static>>) -> Result<()> {
        let serial = self.serial.get_value()?;
        let email = self.email.get_value()?;
        let email_passwd = self.email_passwd.get_value()?;
        let wifi_enable = self.wifi_enable.get_value()?;

        let mut user = User::default();
        user.set_email(email.as_str());
        user.set_password(EncryptGeneric::get_sha256(email_passwd.to_bytes())?.as_str());
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

        let enable_dst = self.enable_dst.get_value().unwrap_or_default();
        DateTime::set_daylight_saving_time(enable_dst);
        self.config.get_daylight_saving_time().set_enabled(enable_dst);

        self.config.set_serial(&Bytes::from_as_sync_str(&serial));

        self.config.apply_locale();
        self.config.apply_daylight_saving_time();
        self.config.apply_ntp();
        self.config.apply_wifi();
        self.config.apply_session();
        Config::save()?;

        Ok(())
    }

    pub(super) fn new() -> Self {
        Self {
            config: Config::shared(),
            fsm_state: FSMState::Serial,
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
