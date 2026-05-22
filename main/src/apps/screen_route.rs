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

mod set_config;
mod main;

use alloc::boxed::Box;
use alloc::sync::Arc;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;


use set_config::ScreenSetConfig;
use crate::apps::screen_route::main::ScreenMain;
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
            FSMState::MenuInfo                  => self.handle_menu_info(lcd, display_signal, status_signal, rtc),
            FSMState::MenuDateTime              => self.handle_menu_date_time(lcd, display_signal, status_signal, rtc),
            FSMState::MenuDaylightSavingTime    => self.handle_menu_daylight_saving_time(lcd, display_signal, status_signal, rtc),
            FSMState::MenuWifi                  => self.handle_menu_wifi(lcd, display_signal, status_signal, rtc),
            FSMState::MenuUser                  => self.handle_menu_user(lcd, display_signal, status_signal, rtc),
        }

        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

impl ScreenRoute {
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
        } else if let Some(screen) = &mut self.current_screen {
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
        } else if let Some(screen) = &mut self.current_screen {
            if screen.draw(lcd, display_signal, status_signal, rtc).is_ok() {
                if let Some(screen_main) = screen.as_any_mut().downcast_mut::<ScreenMain>() {
                    self.check_staus_counter = 0;
                    //let _selected_screen = screen_main.get_selected_screen();
                }
                self.current_screen = None;
            }
        }
    }

    fn handle_menu_info(&mut self,
        _lcd: &mut dyn LCDDisplayFn,
        _display_signal: &mut EventBits,
        _status_signal: &mut EventBits,
        _rtc: &Arc<Mutex<dyn RTC + 'static>>
    ) { 
        todo!() 
    }
    
    fn handle_menu_date_time(&mut self,
        _lcd: &mut dyn LCDDisplayFn,
        _display_signal: &mut EventBits,
        _status_signal: &mut EventBits,
        _rtc: &Arc<Mutex<dyn RTC + 'static>>
    ) { 
        todo!() 
    }
    
    fn handle_menu_daylight_saving_time(&mut self,
        _lcd: &mut dyn LCDDisplayFn,
        _display_signal: &mut EventBits,
        _status_signal: &mut EventBits,
        _rtc: &Arc<Mutex<dyn RTC + 'static>>
    ) { 
        todo!() 
    }
    
    fn handle_menu_wifi(&mut self,
        _lcd: &mut dyn LCDDisplayFn,
        _display_signal: &mut EventBits,
        _status_signal: &mut EventBits,
        _rtc: &Arc<Mutex<dyn RTC + 'static>>
    ) { 
        todo!() 
    }
    
    fn handle_menu_user(&mut self,
        _lcd: &mut dyn LCDDisplayFn,
        _display_signal: &mut EventBits,
        _status_signal: &mut EventBits,
        _rtc: &Arc<Mutex<dyn RTC + 'static>>
    ) { 
        todo!() 
    }

    pub const fn new() -> Self {
        Self {
            fsm_state: FSMState::Init,
            check_staus_counter: 0,
            current_screen: None,
        }
    }
}