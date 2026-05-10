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

use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Result, bytes_to_hex};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::config::Config;
use crate::apps::display::check::Check;
use crate::apps::display::input::Input;
use crate::apps::display::date::Date;
use crate::apps::display::select::Select;
use crate::apps::display::time::Time;

use crate::apps::session::User;
use crate::apps::signals::display::DisplayFlag;
use crate::drivers::date_time::DateTime;
use crate::drivers::platform::Hardware;
use crate::drivers::wifi::Auth;
use crate::traits::hardware::HardwareFn;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute, ScreenSelections, screen_selections_new};

static mut FSM_STATE: FSMState = FSMState::Serial;
static mut OLD_FSM_STATE: FSMState = FSMState::Serial;
static UPDATE_DRAW: AtomicBool = AtomicBool::new(false);

impl Auth {
    fn as_bytes(&self) -> Bytes<{DISPLAY_INPUT_MAX_SIZE}> {
        match self {
            Self::Open => Bytes::from_str("OPEN"),
            Self::Wpa => Bytes::from_str("WPA"),
            Self::Wpa2 => Bytes::from_str("WPA2"),
            Self::Wpa2Mixed => Bytes::from_str("WPA2 MIXED"),
            Self::Wpa3 => Bytes::from_str("WPA3"),
            Self::Wpa2Wpa3 => Bytes::from_str("WPA3/WPA2"),
            _ => Bytes::default(),
        }
    }

    fn fill_screen_selections() -> ScreenSelections {
        let mut selections = screen_selections_new();
        selections[0] = Self::Open.as_bytes();
        selections[1] = Self::Wpa.as_bytes();
        selections[2] = Self::Wpa2.as_bytes();
        selections[3] = Self::Wpa2Mixed.as_bytes();
        selections[4] = Self::Wpa3.as_bytes();
        selections[5] = Self::Wpa2Wpa3.as_bytes();
        selections
    }

}

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
        date_time: &DateTime
        
    ) -> Result<()> {

        if UPDATE_DRAW.load(Ordering::SeqCst) {
            UPDATE_DRAW.store(false, Ordering::SeqCst);
            *display_signal |= DisplayFlag::Draw as u32;
        }

        let fsm_state = unsafe { *&raw const FSM_STATE };
        
        match fsm_state {
            FSMState::Serial => {
                

                let unique_id = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str(bytes_to_hex(&Hardware::get_unique_id()).as_str());
                let unique_id = Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_bytes(&unique_id[..(unique_id.len()/3) * 2]);

                let mut param = ScreenParam::default();
                param.input = Some(unique_id);


                self.serial.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
                    &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert Serial Number"), 
                    param, 
                    Some(|_, confirmed| {
                        unsafe { OLD_FSM_STATE = FSM_STATE; }
                        if confirmed {
                            unsafe { FSM_STATE = FSMState::Email; }
                        } 
                        UPDATE_DRAW.store(true, Ordering::SeqCst);
                    })
                )?;
            }
            FSMState::Email => {
                let mut param = ScreenParam::<u16>::default();
                param.input = Some(Bytes::from_as_sync_str(self.config.get_session().get_user_local().get_email()));

                self.email.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
                    &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert Email"), 
                    param, 
                    Some(|_, confirmed| {
                        unsafe { OLD_FSM_STATE = FSM_STATE; }
                        if confirmed {
                            unsafe { FSM_STATE = FSMState::EmailPasswd; }
                        } 
                        UPDATE_DRAW.store(true, Ordering::SeqCst);
                    })
                )?;
            }
            FSMState::EmailPasswd => {
                let mut param = ScreenParam::<u16>::default();
                param.input = Some(Bytes::from_as_sync_str(self.config.get_session().get_user_local().get_password()));
                param.input_secret_mode = Some(true);

                self.email_passwd.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
                    &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Insert Password"), 
                    param, 
                    Some(|_, confirmed| {
                        unsafe { OLD_FSM_STATE = FSM_STATE; }
                        if confirmed {
                            unsafe { FSM_STATE = FSMState::EnableWifi; }
                        } 
                        UPDATE_DRAW.store(true, Ordering::SeqCst);
                    })
                )?;
            }
            FSMState::EnableWifi => {
                let mut param = ScreenParam::default();
                param.check = Some(self.wifi_enable.get_value().unwrap_or(false));

                self.wifi_enable.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
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
                            // Skip WiFi config and go to next screen
                            unsafe { FSM_STATE = FSMState::Serial; }
                        }
                        UPDATE_DRAW.store(true, Ordering::SeqCst);
                    })
                )?;
            }
            FSMState::Ssid => {
                let mut param = ScreenParam::default();
                param.input = Some(Bytes::from_as_sync_str(self.config.get_wifi_config().get_ssid()));

                self.wifi_ssid.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
                    &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi SSID"), 
                    param, 
                    Some(|_, confirmed| {
                        unsafe { OLD_FSM_STATE = FSM_STATE; }
                        if confirmed {
                            unsafe { FSM_STATE = FSMState::Passwd; }
                        } else {
                            // Skip WiFi config and go to next screen
                            unsafe { FSM_STATE = FSMState::EnableWifi; }
                        }
                        UPDATE_DRAW.store(true, Ordering::SeqCst);
                    })
                )?;
            }
            FSMState::Passwd => {
                let param = ScreenParam::default();

                self.wifi_passwd.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
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
                    })
                )?;
            }
            FSMState::Auth => {
                let mut param = ScreenParam::default();
                param.selects = Some(Auth::fill_screen_selections());

                self.auth.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
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
                    })
                )?;
            }
            FSMState::Date => {
                let mut param = ScreenParam::default();
                param.date_time = Some(date_time.clone());

                self.date.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
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
                    })
                )?;
            }
            FSMState::Time => {
                let mut param = ScreenParam::default();
                param.date_time = Some(date_time.clone());

                self.time.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
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
                    })
                )?;
            }
            FSMState::EnableDst => {
                let mut param = ScreenParam::default();
                param.check = Some(self.enable_dst.get_value().unwrap_or(false));
    
                

                self.enable_dst.draw(
                    lcd, 
                    display_signal, 
                    date_time, 
                    &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Enable DST?"), 
                    param, 
                    Some(move |_, confirmed| {
                        if confirmed {
                            // Configuration complete, go back to menu or next step
                            unsafe { FSM_STATE = FSMState::SetConfig; }
                        } else {
                            unsafe { FSM_STATE = OLD_FSM_STATE; }
                            
                        }
                        UPDATE_DRAW.store(true, Ordering::SeqCst);
                    })
                )?;
            }
            FSMState::SetConfig => {
                let serial = self.serial.get_value().unwrap_or(Bytes::new());
                let email = self.email.get_value().unwrap_or(Bytes::new());
                let email_passwd = self.email_passwd.get_value().unwrap_or(Bytes::new());
                let wifi_enable = self.wifi_enable.get_value().unwrap_or(false);
                let wifi_ssid = self.wifi_ssid.get_value().unwrap_or(Bytes::new());
                let wifi_passwd = self.wifi_passwd.get_value().unwrap_or(Bytes::new());
                let auth = match self.auth.get_value() {
                    Ok(values) => values
                        .iter()
                        .find(|value| !value.is_empty())
                        .copied()
                        .unwrap_or(Bytes::<DISPLAY_INPUT_MAX_SIZE>::new()),
                    Err(_) => Bytes::<DISPLAY_INPUT_MAX_SIZE>::new(),
                };
                    
                    // self.date.get_value().unwrap_or_default();
                    // self.time.get_value().unwrap_or_default();

                
                
                let mut user = User::default();
                user.set_email(email.as_str());
                user.set_password(email_passwd.as_str());
                self.config.get_session().set_user(&user);
                
                if wifi_enable {
                    self.config.get_wifi_config().set_ssid(wifi_ssid.as_str());
                    self.config.get_wifi_config().set_password(wifi_passwd.as_str());
                    self.config.get_wifi_config().set_auth(match auth.as_str() {
                        "OPEN" => Auth::Open,
                        "WPA" => Auth::Wpa,
                        "WPA2" => Auth::Wpa2,
                        "WPA2 MIXED" => Auth::Wpa2Mixed,
                        "WPA3" => Auth::Wpa3,
                        "WPA3/WPA2" => Auth::Wpa2Wpa3,
                        _ => Auth::Open,
                    });
                } else {
                    self.config.get_wifi_config().set_ssid("");
                    self.config.get_wifi_config().set_password("");
                    self.config.get_wifi_config().set_auth(Auth::Open);
                    self.config.get_ntp_config_mut().set_server("");
                    self.config.get_ntp_config_mut().set_port(0);
                    self.config.get_ntp_config_mut().set_msg_len(0);
                }

                DateTime::set_daylight_saving_time(self.enable_dst.get_value().unwrap_or_default());
                
                self.config.set_serial(&Bytes::from_as_sync_str(&serial));

                unsafe { OLD_FSM_STATE = FSMState::SetConfig; }
                unsafe { FSM_STATE = FSMState::End; }
            }
            FSMState::End => (),
        }


        Ok(())
    }
}

impl ScreenSetConfig {
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