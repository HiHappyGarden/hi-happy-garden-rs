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

#![allow(dead_code)]

mod info;
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
use crate::apps::screen_route::info::ScreenInfo;
use crate::apps::screen_route::main::ScreenMain;
use crate::apps::screen_route::date_time::ScreenDateTime;
use crate::apps::screen_route::daylight_saving_time::ScreenDaylightSavingTime;
use crate::apps::screen_route::wifi::ScreenWifi;
use crate::apps::screen_route::user::ScreenUser;
use crate::apps::signals::display::DisplayFlag;
use crate::apps::signals::status::StatusFlag;
use crate::traits::rtc::RTC;
use crate::traits::screen::ScreenRoute as ScreenRouteFn;
use crate::traits::lcd_display::LCDDisplayFn;

pub static mut SCREEN_ROUTE: ScreenRoute = ScreenRoute::new();

const CHECK_STATUS_THRESHOLD: u8 = 5;

enum FSMState {
    Init,
    SetConfig,
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
            2 => FSMState::Menu,
            3 => FSMState::MenuInfo,
            4 => FSMState::MenuDateTime,
            5 => FSMState::MenuDaylightSavingTime,
            6 => FSMState::MenuWifi,
            7 => FSMState::MenuUser,
            _ => FSMState::Init, // Default case
        }
    }
}

impl From<FSMState> for i8 {
    fn from(state: FSMState) -> Self {
        match state {
            FSMState::Init => 0,
            FSMState::SetConfig => 1,
            FSMState::Menu => 2,
            FSMState::MenuInfo => 3,
            FSMState::MenuDateTime => 4,
            FSMState::MenuDaylightSavingTime => 5,
            FSMState::MenuWifi => 6,
            FSMState::MenuUser => 7,
        }
    }
}

pub struct ScreenRoute {
    fsm_state: FSMState,
    check_staus_counter: u8,
    current_screen: Option<Box<dyn ScreenRouteFn>>,
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
            FSMState::Menu                      => self.handle_menu(lcd, display_signal, status_signal, rtc),
            FSMState::MenuInfo                  => self.handle_submenu(lcd, display_signal, status_signal, rtc, || Box::new(ScreenInfo::new())),
            FSMState::MenuDateTime              => self.handle_submenu(lcd, display_signal, status_signal, rtc, || Box::new(ScreenDateTime::new())),
            FSMState::MenuDaylightSavingTime    => self.handle_submenu(lcd, display_signal, status_signal, rtc, || Box::new(ScreenDaylightSavingTime::new())),
            FSMState::MenuWifi                  => self.handle_submenu(lcd, display_signal, status_signal, rtc, || Box::new(ScreenWifi::new())),
            FSMState::MenuUser                  => self.handle_submenu(lcd, display_signal, status_signal, rtc, || Box::new(ScreenUser::new())),
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

    fn handle_menu(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        status_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) {
        if self.current_screen.is_none() {
            self.current_screen = Some(Box::new(ScreenMain::new()));
        }
        if let Some(screen) = &mut self.current_screen {
            if screen.draw(lcd, display_signal, status_signal, rtc).is_ok() {
                if let Some(screen_main) = screen.as_any_mut().downcast_mut::<ScreenMain>() {

                    let mut main_selected_screen: i8 = (screen_main.get_selected_screen() as Option<main::FSMState>).unwrap_or(main::FSMState::Info).into();
                    main_selected_screen += 1;
                    self.fsm_state = FSMState::from(main_selected_screen + <FSMState as Into<i8>>::into(FSMState::Menu));
                    self.current_screen = None;
                    self.check_staus_counter = 0;
                    // Clear button events so the incoming screen does not see the
                    // same (or bounced) press that triggered this transition.
                    *display_signal &= !Self::BUTTON_MASK;      
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
        build_screen: fn() -> Box<dyn ScreenRouteFn>,
    ) {
        if self.current_screen.is_none() {
            self.current_screen = Some(build_screen());
            *display_signal &= !Self::BUTTON_MASK;
            *display_signal |= DisplayFlag::Draw as u32;
        }
        if let Some(screen) = &mut self.current_screen {
            if screen.draw(lcd, display_signal, status_signal, rtc).is_ok() {
                self.current_screen = None;
                self.fsm_state = FSMState::Menu;
            }
        }
    }

    pub const fn new() -> Self {
        Self {
            fsm_state: FSMState::Init,
            check_staus_counter: 0,
            current_screen: None,
        }
    }
}