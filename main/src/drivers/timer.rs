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

use core::any::Any;
use core::ffi::c_void;

use osal_rs::utils::Result;

use crate::drivers::pico::hw_timer::TIMER_FN;



 pub(in crate::drivers) struct TimerFn {
    pub(in crate::drivers) add_repeating_ms: fn (delay_ms: i32, user_data: &dyn Any, callback: extern "C" fn(*mut c_void)) -> Result<Timer>,
    pub(in crate::drivers) cancel: fn (timer: Timer)
}

pub struct Timer {
    instance: *mut c_void,
}

impl Timer {
    pub(in crate::drivers) fn new(instance: *mut c_void) -> Self {
        Timer { 
            instance 
        }
    }

    pub(in crate::drivers) fn get_instance(&self) -> *mut c_void {
        self.instance
    }

    pub fn add_repeating_ms(delay_ms: i32, user_data: &dyn Any, callback: extern "C" fn(*mut c_void)) -> Result<Self> {
        (TIMER_FN.add_repeating_ms)(delay_ms, user_data, callback)
    }

    pub fn cancel(self) {
        (TIMER_FN.cancel)(self);
    }
}