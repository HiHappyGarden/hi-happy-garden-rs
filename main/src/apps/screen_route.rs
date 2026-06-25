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



mod info;
mod login;
mod set_config;
mod main;
mod date_time;
mod daylight_saving_time;
mod wifi;
mod user;
mod auth;

use core::any::Any;

use alloc::boxed::Box;
use alloc::sync::Arc;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;


use set_config::ScreenSetConfig;
use crate::apps::config::Config;
use crate::apps::screen_route::info::ScreenInfo;
use crate::apps::screen_route::login::ScreenLogin;
use crate::apps::screen_route::main::{ScreenMain, FSMState as MainFSMState};
use crate::apps::screen_route::date_time::ScreenDateTime;
use crate::apps::screen_route::daylight_saving_time::ScreenDaylightSavingTime;
use crate::apps::screen_route::wifi::ScreenWifi;
use crate::apps::screen_route::user::ScreenUser;
use crate::apps::signals::display::{DisplayFlag, DisplaySignal};
use crate::apps::signals::status::StatusFlag;
use crate::traits::rtc::RTC;
use crate::traits::screen::ScreenRoute as ScreenRouteFn;
use crate::traits::signal::Signal;
use crate::traits::lcd_display::LCDDisplayFn;


pub(in crate::apps) static mut SCREEN_ROUTE: ScreenRoute = ScreenRoute::new();

const CHECK_STATUS_THRESHOLD: u8 = 5;

enum FSMState {
    Init,
    SetConfig,
    Login,
    Menu,
    MenuInfo,
    MenuDateTime,
    MenuDaylightSavingTime,
    MenuWifi,
    MenuUser,
}

impl From<i8> for FSMState {
    fn from(value: i8) -> Self {
        match value {
            0 => FSMState::Init,
            1 => FSMState::SetConfig,
            2 => FSMState::Login,
            3 => FSMState::Menu,
            4 => FSMState::MenuInfo,
            5 => FSMState::MenuDateTime,
            6 => FSMState::MenuDaylightSavingTime,
            7 => FSMState::MenuWifi,
            8 => FSMState::MenuUser,
            _ => FSMState::Init, // Default case
        }
    }
}

impl From<FSMState> for i8 {
    fn from(state: FSMState) -> Self {
        match state {
            FSMState::Init => 0,
            FSMState::SetConfig => 1,
            FSMState::Login => 2,
            FSMState::Menu => 3,
            FSMState::MenuInfo => 4,
            FSMState::MenuDateTime => 5,
            FSMState::MenuDaylightSavingTime => 6,
            FSMState::MenuWifi => 7,
            FSMState::MenuUser => 8,
        }
    }
}

pub(in crate::apps) struct ScreenRoute {
    config: &'static mut Config,
    fsm_state: FSMState,
    main_fsm_state: MainFSMState,
    check_staus_counter: u8,
    current_screen: Option<Box<dyn ScreenRouteFn>>,
    has_local_user: bool,
}

impl ScreenRouteFn for ScreenRoute {
    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits, 
        status_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> osal_rs::utils::Result<()> {
        
        match self.fsm_state {
            FSMState::Init                      => self.handle_init(status_signal),
            FSMState::SetConfig                 => self.handle_set_config(lcd, display_signal, status_signal, rtc),
            FSMState::Login                     => self.handle_login(lcd, display_signal, status_signal, rtc),
            FSMState::Menu                      => self.handle_menu(lcd, display_signal, status_signal, rtc),
            FSMState::MenuInfo                  => self.handle_submenu(lcd, display_signal, status_signal, rtc, MainFSMState::Info, || Box::new(ScreenInfo::new()) ),
            FSMState::MenuDateTime              => self.handle_submenu(lcd, display_signal, status_signal, rtc, MainFSMState::DateTime, || Box::new(ScreenDateTime::new())),
            FSMState::MenuDaylightSavingTime    => self.handle_submenu(lcd, display_signal, status_signal, rtc, MainFSMState::DaylightSavingTime, || Box::new(ScreenDaylightSavingTime::new())),
            FSMState::MenuWifi                  => self.handle_submenu(lcd, display_signal, status_signal, rtc, MainFSMState::Wifi, || Box::new(ScreenWifi::new())),
            FSMState::MenuUser                  => self.handle_submenu(lcd, display_signal, status_signal, rtc, MainFSMState::User, || Box::new(ScreenUser::new())),
        }

        Ok(())
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

impl ScreenRoute {

    const BUTTON_MASK: u32 = DisplayFlag::ButtonPressed as u32
                | DisplayFlag::ButtonReleased as u32
                | DisplayFlag::EncoderButtonPressed as u32
                | DisplayFlag::EncoderButtonReleased as u32;

    #[inline]
    fn request_redraw(display_signal: &mut EventBits) {
        *display_signal |= DisplayFlag::Draw as u32;
        DisplaySignal::set(DisplayFlag::Draw as u32);
    }

    fn handle_init(&mut self, status_signal: &mut EventBits) {
        if StatusFlag::CheckConfig.check_signal(*status_signal) {
            self.check_staus_counter += 1;
            if self.check_staus_counter >= CHECK_STATUS_THRESHOLD {
                self.fsm_state = FSMState::SetConfig;
                self.check_staus_counter = 0;
            }
        } else if StatusFlag::Ready.check_signal(*status_signal) {
            self.check_staus_counter += 1;
            if self.check_staus_counter >= CHECK_STATUS_THRESHOLD {
                self.fsm_state = FSMState::Menu;
                self.check_staus_counter = 0;
            }
        } else {
            self.check_staus_counter = 0;
        }
    }

    fn handle_set_config(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        status_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) {
        if self.current_screen.is_none() {
            self.current_screen = Some(Box::new(ScreenSetConfig::new()));
        }
        if let Some(screen) = &mut self.current_screen {
            if screen.draw(lcd, display_signal, status_signal, rtc).is_ok() {
                self.current_screen = None;
                self.fsm_state = FSMState::Menu;
            }
        }
    }

    fn handle_login(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        status_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>
    ) {
        if self.current_screen.is_none() {
            self.current_screen = Some(Box::new(ScreenLogin::new()));
        }
        if let Some(screen) = &mut self.current_screen {
            if screen.draw(lcd, display_signal, status_signal, rtc).is_ok() {
                self.current_screen = None;
                self.fsm_state = FSMState::Init;
            }
        }
        
    }

    fn handle_menu(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        status_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) {
        if self.current_screen.is_none() {
            self.current_screen = Some(Box::new(ScreenMain::new(self.main_fsm_state)));
        }
        if let Some(screen) = &mut self.current_screen {
            if screen.draw(lcd, display_signal, status_signal, rtc).is_ok() {
                if let Some(screen_main) = screen.as_any_mut().downcast_mut::<ScreenMain>() {

                    if self.config.get_session().is_set_user_local() {
                        self.has_local_user = true;
                    } else {
                        self.has_local_user = false;
                    }

                    let mut main_selected_screen: i8 = (screen_main.get_selected_screen() as Option<main::FSMState>).unwrap_or(main::FSMState::Info).into();
                    main_selected_screen += 1;
                    self.fsm_state = FSMState::from(main_selected_screen + <FSMState as Into<i8>>::into(FSMState::Menu));
                    self.current_screen = None;
                    self.check_staus_counter = 0;
                    // Clear button events so the incoming screen does not see the
                    // same (or bounced) press that triggered this transition.
                    *display_signal &= !Self::BUTTON_MASK;
                    Self::request_redraw(display_signal);
                }
            }
        }
    }

    fn handle_submenu(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        status_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
        back: MainFSMState,
        build_screen: fn() -> Box<dyn ScreenRouteFn>,
    ) {
        if self.has_local_user && !StatusFlag::UserLogged.check_signal(*status_signal) && MainFSMState::Info != back {
            self.main_fsm_state = back;
            self.fsm_state = FSMState::Login;
            Self::request_redraw(display_signal);
            return;
        }

        if self.current_screen.is_none() {
            self.current_screen = Some(build_screen());
            *display_signal &= !Self::BUTTON_MASK;
            *display_signal |= DisplayFlag::Draw as u32;
        }
        if let Some(screen) = &mut self.current_screen {
            if screen.draw(lcd, display_signal, status_signal, rtc).is_ok() {
                self.current_screen = None;
                self.main_fsm_state = back;
                self.fsm_state = FSMState::Menu;
                Self::request_redraw(display_signal);
            }
        }
    }

    pub(in crate::apps) const fn new() -> Self {
        Self {
            config: Config::shared(),
            fsm_state: FSMState::Init,
            main_fsm_state: MainFSMState::Info,
            check_staus_counter: 0,
            has_local_user: false,
            current_screen: None,
        }
    }
}