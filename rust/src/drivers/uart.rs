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

use core::ptr::addr_of_mut;
use core::time::Duration;

use osal_rs::{log_error, minimal_stack_size};
use osal_rs::os::{Queue, QueueFn, Thread, ThreadFn};
use osal_rs::os::types::{TickType, UBaseType};
use osal_rs::utils::{Bytes, Error, Ptr, Result};

use crate::traits::rx_tx::OnReceive; 
use crate::traits::state::Initializable;
use crate::drivers::platform::{UART_FN, UART_CONFIG, ThreadPriority}; 

const APP_TAG: &str = "Uart";
const APP_THREAD_NAME: &str = "uart_trd";
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

const UART_QUEUE_SIZE: UBaseType = 64;
static mut UART_QUEUE: Option<Queue> = None;

const fn uart_queue() -> &'static Queue {
    unsafe {
        match &*&raw const UART_QUEUE {
            Some(queue) => queue,
            None => panic!("UART_QUEUE is not initialized"),    
        }
    }
}

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
    pub receive: Option<&'static Queue>,
    pub deinit: fn(&UartConfig) -> Result<()>,
}

#[derive(Clone)]
pub struct Uart {
    functions: &'static UartFn,
    config: &'static UartConfig,
    thread: Thread,
}

unsafe impl Sync for Uart {}
unsafe impl Send for Uart {}

impl Initializable for Uart {
    fn init(&mut self) -> Result<()> {
        if (self.functions.init)(&self.config).is_err() {
            return Err(Error::Unhandled("Failed to initialize Uart"))
        }

        if let Ok(queue) =  Queue::new(UART_QUEUE_SIZE, 1) {
            unsafe {
                UART_QUEUE = Some(queue);
            }
        } else {
            log_error!(APP_TAG, "Error creating UART queue");
            return Err(Error::OutOfMemory)
        }

        unsafe {
            UART_FN.receive = Some(uart_queue());
        };

        Ok(())
    }
}

impl Uart {
    pub fn new() -> Self {
        Self { 
            functions: unsafe {
                &mut *addr_of_mut!(UART_FN)   
            },
            config: unsafe {
                &mut *addr_of_mut!(UART_CONFIG)   
            }, 
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, minimal_stack_size!(), ThreadPriority::Normal),
        }
    }

    
    pub fn transmit(&self, data: &[u8]) -> usize {
        (self.functions.transmit)(data)
    }

    pub fn add_listener(&mut self, listener: &'static dyn OnReceive) {


        let ret = self.thread.spawn_simple(move || {

            
            
            loop {
                let mut bytes = [0u8; UART_QUEUE_SIZE as usize];
                uart_queue().fetch(&mut bytes, TickType::MAX).unwrap();

                listener.on_receive(SOURCE, &bytes);
            }

        });

        if let Err(e) = ret {
            log_error!(APP_TAG, "Error spawning encoder thread: {:?}", e);
        }
    }
}