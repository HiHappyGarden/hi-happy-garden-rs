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

use osal_rs::log_info;
use osal_rs::utils::{Ptr, Result};

use crate::traits::rx_tx::{OnReceive, SetTransmit}; 
use crate::traits::state::Initializable;
use crate::drivers::platform::{UART_FN, UART_CONFIG}; 

const APP_TAG: &str = "Uart";

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UartParity {
    None,
    Even,
    Odd,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UartStopBits {
    Half,
    One,
    OneAndHalf,
    Two,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UartDataBits {
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UartFlowControl {
    None,
    RtsCts,
    XonXoff,
}

// const UART_QUEUE_SIZE: UBaseType = 8;
// static mut UART_QUEUE: Option<Queue> = None;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(in crate::drivers) struct UartConfig {
    pub(in crate::drivers) name: &'static str,
    pub(in crate::drivers) base: Ptr,
    pub(in crate::drivers) baudrate: u32,
    pub(in crate::drivers) data_bits: UartDataBits,
    pub(in crate::drivers) stop_bits: UartStopBits,
    pub(in crate::drivers) parity: UartParity,
    pub(in crate::drivers) flow_control: UartFlowControl,
}

unsafe impl Sync for UartConfig {}
unsafe impl Send for UartConfig {}

#[allow(dead_code)]
#[derive(Clone)]
pub(in crate::drivers) struct UartFn {
    pub(in crate::drivers) init: fn(&UartConfig) -> Result<()>,
    pub(in crate::drivers) transmit: fn(data: &[u8]) -> usize,
    pub(in crate::drivers) add_listener: Option<&'static dyn OnReceive>,
    pub(in crate::drivers) deinit: fn(&UartConfig) -> Result<()>,
}

#[derive(Clone)]
pub struct Uart {
    config: &'static UartConfig,
}

unsafe impl Sync for Uart {}
unsafe impl Send for Uart {}

impl Initializable for Uart {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init uart");
        unsafe {
            (UART_FN.init)(&self.config)?;
        }
        Ok(())
    }
}


impl SetTransmit for Uart {
    fn transmit(&self, data: &[u8]) -> usize {
        unsafe {
            (UART_FN.transmit)(data)
        }
    }
}

impl Uart {
    pub fn shared() -> Self {
        Self { 
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