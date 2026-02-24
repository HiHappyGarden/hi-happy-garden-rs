/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

#![allow(dead_code)]

use osal_rs::access_static_option;
use osal_rs::os::types::{self, EventBits};
use osal_rs::os::{EventGroup, EventGroupFn};
use osal_rs::utils::Result;

static mut DISPLAY_SIGNAL: Option<EventGroup> = None;

pub struct DisplaySignal;


impl DisplaySignal {
    pub fn init() -> Result<()> {
        unsafe {
            DISPLAY_SIGNAL = Some(EventGroup::new()?)
        }
        Ok(())
    }

    pub fn get() -> EventBits {
        access_static_option!(DISPLAY_SIGNAL).get()
    }

    pub fn get_form_isr() -> EventBits {
        access_static_option!(DISPLAY_SIGNAL).get_from_isr()
    }
    
    fn set(bits: EventBits) -> EventBits {
        access_static_option!(DISPLAY_SIGNAL).set(bits)
    }
    
    fn set_from_isr(bits: EventBits) -> Result<()> {
        access_static_option!(DISPLAY_SIGNAL).set_from_isr(bits)
    }
    
    fn clear(bits: EventBits) -> EventBits {
        access_static_option!(DISPLAY_SIGNAL).clear(bits)
    }
    
    fn clear_from_isr(bits: EventBits) -> Result<()> {
        access_static_option!(DISPLAY_SIGNAL).clear_from_isr(bits)
    }
    
    fn wait(mask: EventBits, timeout_ticks: types::TickType) -> EventBits {
        access_static_option!(DISPLAY_SIGNAL).wait(mask, timeout_ticks)
    }

}