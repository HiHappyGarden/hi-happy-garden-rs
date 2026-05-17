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
            FSMState::Init => {
                // Handle Init state
                if StatusFlag::CheckConfig.check_signal(*status_signal) {
                    self.check_staus_counter += 1;
                    if self.check_staus_counter >= CHECK_STATUS_THRESHOLD {
                        self.fsm_state = FSMState::SetConfig;
                        self.check_staus_counter = 0;
                    }
                } else {
                    self.check_staus_counter = 0;
                }
            }
            FSMState::SetConfig => {
                if self.current_screen.is_none() {
                    self.current_screen = Some(Box::new(ScreenSetConfig::new()));
                } else {
                    if let Some(screen) = &mut self.current_screen {
                        screen.draw(lcd, display_signal, status_signal, rtc)?;
                    }
                }

            }
            FSMState::Menu => {
                // Handle Menu state
            }
        }

        Ok(())
    }
}

impl ScreenRoute {
    pub const fn new() -> Self {
        Self {
            fsm_state: FSMState::Init,
            check_staus_counter: 0,
            current_screen: None,
        }
    }
}