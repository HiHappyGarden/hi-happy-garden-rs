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

use osal_rs::os::types::EventBits;


use crate::apps::signals::status::StatusFlag;
use crate::drivers::date_time::DateTime;
use crate::traits::screen::{ScreenRoute as ScreenRouteFn};
use crate::traits::lcd_display::LCDDisplayFn;

pub static mut SCREEN_ROUTE: ScreenRoute = ScreenRoute::new();


enum FSMState {
    Idle,
    Active,
    Confirming,
}

pub struct ScreenRoute(StatusFlag);

impl ScreenRouteFn for ScreenRoute {
    fn draw(&mut self, 
        lcd: &mut impl LCDDisplayFn,
        display_signal: &mut EventBits, 
        status_signal: &EventBits, 
        date_time: &DateTime
    ) -> osal_rs::utils::Result<()> {
        
        Ok(())
    }
}

impl ScreenRoute {
    pub const fn new() -> Self {
        Self(StatusFlag::None)
    }
}