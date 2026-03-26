/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
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

use osal_rs::os::types::{EventBits, TickType};
use osal_rs::utils::Result;



pub trait Signal {

    fn get() -> EventBits;

    fn get_from_isr() -> EventBits;
    
    fn set(bits: EventBits) -> EventBits;

    fn set_from_isr(bits: EventBits) -> Result<()>;
    
    fn clear(bits: EventBits) -> EventBits;
    
    fn clear_from_isr(bits: EventBits) -> Result<()>;
    
    fn wait(mask: EventBits, timeout_ticks: TickType) -> EventBits;
}

/// Macro to generate the boilerplate implementation of a Signal
/// 
/// # Example
/// ```
/// define_signal!(MySignal, MY_SIGNAL);
/// ```
#[macro_export]
macro_rules! define_signal {
    ($signal_name:ident, $static_name:ident) => {
        use osal_rs::os::EventGroupFn;
        
        static mut $static_name: Option<osal_rs::os::EventGroup> = None;

        pub struct $signal_name;

        impl $crate::traits::signal::Signal for $signal_name {
            fn get() -> osal_rs::os::types::EventBits {
                osal_rs::access_static_option!($static_name).get()
            }

            fn get_from_isr() -> osal_rs::os::types::EventBits {
                osal_rs::access_static_option!($static_name).get_from_isr()
            }
            
            fn set(bits: osal_rs::os::types::EventBits) -> osal_rs::os::types::EventBits {
                osal_rs::access_static_option!($static_name).set(bits)
            }
            
            fn set_from_isr(bits: osal_rs::os::types::EventBits) -> osal_rs::utils::Result<()> {
                osal_rs::access_static_option!($static_name).set_from_isr(bits)
            }
            
            fn clear(bits: osal_rs::os::types::EventBits) -> osal_rs::os::types::EventBits {
                osal_rs::access_static_option!($static_name).clear(bits)
            }
            
            fn clear_from_isr(bits: osal_rs::os::types::EventBits) -> osal_rs::utils::Result<()> {
                osal_rs::access_static_option!($static_name).clear_from_isr(bits)
            }
            
            fn wait(mask: osal_rs::os::types::EventBits, timeout_ticks: osal_rs::os::types::TickType) -> osal_rs::os::types::EventBits {
                osal_rs::access_static_option!($static_name).wait(mask, timeout_ticks)
            }
        }

        impl $signal_name {
            pub fn init() -> osal_rs::utils::Result<()> {
                unsafe {
                    $static_name = Some(osal_rs::os::EventGroup::new()?)
                }
                Ok(())
            }
        }
    };
}