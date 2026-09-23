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
use crate::apps::display::check::Check;
use crate::apps::display::input::Input;
use crate::apps::display::select::Select;
use crate::apps::screen_route::auth::{fill_auth_selections, selected_auth_from_selections};
use crate::apps::screen_route::ScreenId;
use crate::apps::signals::display::request_redraw;
use crate::drivers::wifi::Auth;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Enable,
    Ssid,
    Passwd,
    AuthType,
}

pub(super) struct ScreenWifi {
    fsm_state: FSMState,
    enable:  Check,
    ssid:    Input,
    passwd:  Input,
    auth:    Select,
}

impl ScreenRoute<ScreenId> for ScreenWifi {

    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::Wifi
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        match self.fsm_state {
            FSMState::Enable   => self.draw_enable_state(screen_route_ctx),
            FSMState::Ssid     => self.draw_ssid_state(screen_route_ctx),
            FSMState::Passwd   => self.draw_passwd_state(screen_route_ctx),
            FSMState::AuthType => self.draw_auth_state(screen_route_ctx),
        }
    }
}

impl ScreenWifi {

    #[inline]
    fn set_state(&mut self, display_signal: &mut EventBits, next: FSMState) {
        self.fsm_state = next;
        request_redraw(display_signal);
    }

    fn draw_enable_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        
        match self.enable.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Enable WiFi?"),
            ScreenParam::Check(Config::shared().get_wifi_config().is_enabled()),
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(param) => {

                match param {
                    ScreenParam::Check(_) => {
                        self.set_state(display_signal, FSMState::Ssid);
                        Ok(Nav::Stay)
                    }
                    _ => {
                        self.save()?;
                        Ok(Nav::Pop)
                    }
                }
            }
            Answer::Cancelled => Ok(Nav::Pop),
        }
    }

    fn draw_ssid_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        
        match self.ssid.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi SSID"),
            ScreenParam::Input { value: Bytes::from_as_sync_str(&Config::shared().get_wifi_config().get_ssid()), secret_mode: false }
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.set_state(display_signal, FSMState::Passwd);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => {
                self.set_state(display_signal, FSMState::Enable);
                Ok(Nav::Stay)
            }
        }
    }

    fn draw_passwd_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {

        match self.passwd.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Password"),
            ScreenParam::Input { value: Bytes::from_as_sync_str(&Config::shared().get_wifi_config().get_password()), secret_mode: true },
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.set_state(display_signal, FSMState::AuthType);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => {
                self.set_state(display_signal, FSMState::Ssid);
                Ok(Nav::Stay)
            }
        }
    }

    fn draw_auth_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc }: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {

        match self.auth.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Auth"),
            ScreenParam::Selects(fill_auth_selections(Config::shared().get_wifi_config().get_auth())),
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(_) => {
                self.save()?;
                Ok(Nav::Pop)
            }
            Answer::Cancelled => {
                self.set_state(display_signal, FSMState::Passwd);
                Ok(Nav::Stay)
            }
        }
    }

    fn save(&mut self) -> Result<()> {
        let wifi_config = Config::shared().get_wifi_config();

        if self.enable.get_value().unwrap_or(false) {
            let ssid   = self.ssid.get_value()?;
            let passwd = self.passwd.get_value()?;
            let selected_auth = selected_auth_from_selections(&self.auth.get_value()?);

            wifi_config.set_ssid(ssid.as_str());
            wifi_config.set_password(passwd.as_str());
            wifi_config.set_auth(selected_auth);
            wifi_config.set_enabled(true);
        } else {
            wifi_config.set_ssid("");
            wifi_config.set_password("");
            wifi_config.set_auth(Auth::Open);
            wifi_config.set_enabled(false);
        }

        Config::shared().apply_wifi();
        Config::save()?;
        Ok(())
    }

    pub(super) const fn new() -> Self {
        Self {
            fsm_state: FSMState::Enable,
            enable: Check::new(),
            ssid:   Input::new(),
            passwd: Input::new(),
            auth:   Select::new(),
        }
    }
}
