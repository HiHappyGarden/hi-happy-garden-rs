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
use osal_rs::os::{Mutex, MutexFn, System, SystemFn, ToPriority};
use osal_rs::utils::{ArcMux, Error, Result};

use alloc::rc::Rc;

use alloc::sync::Arc;
use core::cell::RefCell;
use core::ptr::read;

use crate::drivers::gpio;
use crate::drivers::pico::ffi::hhg_cyw43_arch_init;
use crate::drivers::uart::Uart;
use crate::traits::button::{ButtonState, OnClickable, SetClickable as ButtonOnClickable};
use crate::traits::encoder::{OnRotatableAndClickable as EncoderOnRotatableAndClickable, SetRotatableAndClickable};
use crate::traits::hardware::HardwareFn;
use crate::traits::rx_tx::OnReceive;
use super::gpio::{GPIO_FN, GPIO_CONFIG_SIZE};
use crate::traits::state::Initializable;

use crate::drivers::platform::{Button, Encoder, Gpio, GpioConfigs, GpioPeripheral, UART_FN};

const APP_TAG: &str = "Hardware";


#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ThreadPriority {
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

impl ToPriority for ThreadPriority {
    #[inline]
    fn to_priority(&self) -> UBaseType {
        *self as UBaseType
    }
}

#[allow(unused)]
impl ThreadPriority {
    pub fn from_priority(priority: UBaseType) -> Self {
        use ThreadPriority::*;
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
    uart: Uart,
    encoder: Encoder,
    button: Button,
}

impl Initializable for Hardware {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init hardware");

        log_info!(APP_TAG, "Init wifi cyw43");
        let ret = unsafe { hhg_cyw43_arch_init() };
        if ret != 0 {
            log_info!(APP_TAG, "Wi-Fi init failed");
            return Err(Error::ReturnWithCode(ret));
        }

        Gpio::new().init()?;

        self.uart.init()?;

        self.encoder.init()?;

        self.button.init()?;

        log_info!(APP_TAG, "Hardware temperature: {}", self.get_temperature());

        log_info!(APP_TAG, "Hardware initialized successfully heap_free:{}", System::get_free_heap_size());
        Ok(())
    } 
}

impl HardwareFn<'static> for Hardware {
    #[inline]
    fn set_button_handler(&mut self, clickable: &'static dyn OnClickable) {
        self.button.set_on_click(clickable);
    }

    #[inline]
    fn set_encoder_handler(&mut self, rotable_and_clickable: &'static dyn EncoderOnRotatableAndClickable) {
        self.encoder.set_on_rotate_and_click(rotable_and_clickable);
    }
    
    fn get_temperature(&self) -> f32 {
        let gpio = Gpio::new();

        let mut sum = 0f32;
        for _ in 0..Self::SAMPLES {
            let raw_value = gpio.read(&GpioPeripheral::InternalTemp).unwrap_or(0);
            let temp = Self::temperature_conversion(raw_value);
            sum += temp / Self::SAMPLES as f32;
            System::delay(10);
        }
        sum 
    }
}

impl Hardware {
    pub fn new() -> Self {        
        
        Self { 
            uart: Uart::new(),
            encoder: Encoder::new(),
            button: Button::new(),
        }
    }
}