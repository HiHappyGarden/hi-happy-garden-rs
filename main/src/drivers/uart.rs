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

use osal_rs::log_info;
use osal_rs::utils::{Ptr, Result};

use crate::traits::rx_tx::{OnReceive, SetTransmit}; 
use crate::traits::state::Initializable;
use crate::drivers::platform::{UART_FN, UART_CONFIG}; 

const APP_TAG: &str = "Uart";
const SOURCE: &str = "UART";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UartParity {
    None,
    Even,
    Odd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UartStopBits {
    Half,
    One,
    OneAndHalf,
    Two,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UartDataBits {
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UartFlowControl {
    None,
    RtsCts,
    XonXoff,
}

// const UART_QUEUE_SIZE: UBaseType = 8;
// static mut UART_QUEUE: Option<Queue> = None;


#[derive(Clone, Copy)]
pub struct UartConfig {
    pub name: &'static str,
    pub base: Ptr,
    pub baudrate: u32,
    pub data_bits: UartDataBits,
    pub stop_bits: UartStopBits,
    pub parity: UartParity,
    pub flow_control: UartFlowControl,
}

unsafe impl Sync for UartConfig {}
unsafe impl Send for UartConfig {}


#[derive(Clone)]
pub struct UartFn {
    pub init: fn(&UartConfig) -> Result<()>,
    pub transmit: fn(data: &[u8]) -> usize,
    pub add_listener: Option<&'static dyn OnReceive>,
    pub deinit: fn(&UartConfig) -> Result<()>,
}

#[derive(Clone)]
pub struct Uart {
    functions: &'static UartFn,
    config: &'static UartConfig,
}

unsafe impl Sync for Uart {}
unsafe impl Send for Uart {}

impl Initializable for Uart {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init uart");
        
        (self.functions.init)(&self.config)?;

        Ok(())
    }
}


impl SetTransmit for Uart {
    fn transmit(&self, data: &[u8]) -> usize {
        (self.functions.transmit)(data)
    }
}

impl Uart {
    pub fn shared() -> Self {
        Self { 
            functions: unsafe {
                &*&raw mut UART_FN
            },
            config: unsafe {
                &*&raw mut UART_CONFIG
            }
        }
    }

    #[inline]
    pub fn add_listener(&mut self, listener: &'static dyn OnReceive) {
        unsafe {
             UART_FN.add_listener = Some(listener);
        };
    }
}