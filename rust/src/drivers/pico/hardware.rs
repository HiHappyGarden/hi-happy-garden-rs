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

use alloc::boxed::Box;
use osal_rs::{arcmux, log_info};
use osal_rs::os::types::UBaseType;
use osal_rs::os::{Mutex, MutexFn, ToPriority};
use osal_rs::utils::{ArcMux, Result};

use alloc::rc::Rc;

use alloc::sync::Arc;
use core::cell::RefCell;

use crate::drivers::gpio;
use crate::traits::button::{ButtonState, OnClickable, SetClickable as ButtonOnClickable};
use crate::traits::encoder::{OnRotatableAndClickable as EncoderOnRotatableAndClickable, SetRotatableAndClickable};
use crate::traits::hardware::HardwareFn;
use super::gpio::{GPIO_FN, get_gpio_configs, GPIO_CONFIG_SIZE};
use crate::traits::state::Initializable;

use crate::drivers::platform::{Button, Encoder, Gpio, GpioConfigs, GpioPeripheral};

const APP_TAG: &str = "Hardware";


#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OsalThreadPriority {
    None = 0,
    Idle = 1,
    Low = 4,
    BelowNormal = 8,
    Normal = 12,
    AboveNormal = 16,
    BelowHigh = 19,
    High = 23,
    AboveHigh = 27,
    Realtime = 31,
}

impl ToPriority for OsalThreadPriority {
    #[inline]
    fn to_priority(&self) -> UBaseType {
        *self as UBaseType
    }
}

#[allow(unused)]
impl OsalThreadPriority {
    pub fn from_priority(priority: UBaseType) -> Self {
        use OsalThreadPriority::*;
        match priority {
            1 => Idle,
            2..=4 => Low,
            5..=8 => BelowNormal,
            9..=12 => Normal,
            13..=16 => AboveNormal,
            17..=19 => BelowHigh,
            20..=23 => High,
            24..=27 => AboveHigh,
            28..=31 => Realtime,
            _ => None,
        }
    }
}


pub struct Hardware {
    gpio: ArcMux<Gpio<GPIO_CONFIG_SIZE>>,
    encoder: Encoder,
    button: Button,
}

impl Initializable for Hardware {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init hardware");

        self.gpio.lock()?.init()?;

        self.encoder.init(&mut ArcMux::clone(&self.gpio))?;

        self.button.init(&mut ArcMux::clone(&self.gpio))?;

        Ok(())
    } 
}

impl HardwareFn for Hardware {
    #[inline]
    fn set_button_handler(&mut self, clickable: ArcMux<dyn OnClickable>) {
        self.button.set_on_click(clickable);
    }

    #[inline]
    fn set_encoder_handler(&mut self, rotable_and_clickable: ArcMux<dyn EncoderOnRotatableAndClickable>) {
        self.encoder.set_on_rotate_and_click(rotable_and_clickable);
    }
}

impl Hardware {
    pub fn new() -> Self {

        let gpio = arcmux!(Gpio::<GPIO_CONFIG_SIZE>::new(&GPIO_FN, get_gpio_configs()));
        let gpio_clone = ArcMux::clone(&gpio);
        
        Self { 
            gpio,
            encoder: Encoder::new(ArcMux::clone(&gpio_clone)),
            button: Button::new(ArcMux::clone(&gpio_clone)),
        }
        
    }
}