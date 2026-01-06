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

use osal_rs::{arcmux, utils::{ArcMux, AsSyncStr, Ptr, Result}};

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


#[derive(Clone, Copy)]
pub struct UartConfig<'a> {
    pub name : &'a str,
    pub base: Ptr,
    pub baudrate: u32,
    pub data_bits: UartDataBits,
    pub stop_bits: UartStopBits,
    pub parity: UartParity,
    pub flow_control: UartFlowControl,
}

unsafe impl Sync for UartConfig<'_> {}
unsafe impl Send for UartConfig<'_> {}


#[derive(Clone)]
pub struct UartFn {
    pub init: fn(&UartConfig) -> Result<()>,
    pub transmit: fn(data: &[u8]) -> usize,
    pub receive: Option<ArcMux<Uart<'static>>>,
    pub deinit: fn(&UartConfig) -> Result<()>,
}


pub trait OnReceive {
    fn on_receive(&self, data: &[u8]);
    fn get_source(&self) -> &'static str;
}

#[derive(Clone, Copy)]
pub struct Uart<'a> {
    functions: &'a UartFn,
    config: UartConfig<'a>,
}

unsafe impl Sync for Uart<'_> {}
unsafe impl Send for Uart<'_> {}

impl<'a> OnReceive for Uart<'a> {
    fn on_receive(&self, data: &[u8]) {
        // Default implementation does nothing
    }
    
    fn get_source(&self) -> &'static str {
        "Uart"
    }
}

impl<'a> Uart<'a> {
    pub fn new(config: UartConfig<'a>, functions: &'a mut UartFn) -> Self {
        let ret = Self { config, functions };

        functions.receive = Some(arcmux!(ret));

        

        ret
    }
    
}