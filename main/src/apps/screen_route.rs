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

mod config;

use osal_rs::log_debug;
use osal_rs::os::types::EventBits;


use crate::apps::signals::status::StatusFlag;
use crate::drivers::date_time::DateTime;
use crate::traits::screen::{ScreenRoute as ScreenRouteFn};
use crate::traits::lcd_display::LCDDisplayFn;

pub static mut SCREEN_ROUTE: ScreenRoute = ScreenRoute::new();

const CHECK_STATUS_THRESHOLD: u8 = 5;

enum FSMState {
    Init,
    Config,
    Menu,
}
pub struct ScreenRoute{
    fsm_state: FSMState,
    check_staus_counter: u8,
}

impl ScreenRouteFn for ScreenRoute {
    fn draw(&mut self, 
        _lcd: &mut impl LCDDisplayFn,
        _display_signal: &mut EventBits, 
        status_signal: EventBits, 
        _date_time: &DateTime
    ) -> osal_rs::utils::Result<()> {
        
        match self.fsm_state {
            FSMState::Init => {
                // Handle Init state
                if StatusFlag::CheckConfig.check_signal(status_signal) {
                    self.check_staus_counter += 1;
                    if self.check_staus_counter >= CHECK_STATUS_THRESHOLD {
                        self.fsm_state = FSMState::Config;
                        self.check_staus_counter = 0;
                    }
                } else {
                    self.check_staus_counter = 0;
                }
            }
            FSMState::Config => {
                log_debug!("---->", "ScreenRoute: Config state");
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
        }
    }
}