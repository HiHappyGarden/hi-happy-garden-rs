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


use core::any::Any;
use core::ffi::c_void;
use core::ptr::null_mut;

use osal_rs::utils::{Error, Result};

use crate::drivers::pico::ffi::{hhg_add_repeating_timer_ms, hhg_cancel_repeating_timer};
use crate::drivers::timer::{Timer, TimerFn};

pub(in crate::drivers) const TIMER_FN: TimerFn = TimerFn {
    add_repeating_ms,
    cancel
};


fn add_repeating_ms (delay_ms: i32, user_data: &dyn Any, callback: extern "C" fn(*mut c_void)) -> Result<Timer> {
    let mut timer = null_mut();

    let ret = unsafe {
        hhg_add_repeating_timer_ms(delay_ms, callback, (&raw const user_data) as *mut c_void, &mut timer)
    };
    if !ret {
        Err(Error::OutOfMemory)
    } else {
        Ok(Timer::new(timer))
    }
}

fn cancel (timer: Timer) {
    unsafe {
        hhg_cancel_repeating_timer(timer.get_instance());
    }
}