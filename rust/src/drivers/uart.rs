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

use alloc::{boxed::Box, sync::Arc};
use once_cell::race::OnceBox;
use osal_rs::{arcmux, minimal_stack_size, os::{MutexFn, Queue, QueueFn, Thread, ThreadFn, types::{TickType, UBaseType}}, utils::{ArcMux, AsSyncStr, Bytes, Error, Ptr, Result}};
use osal_rs_tests::freertos::queue_tests;

use crate::{drivers::platform::ThreadPriority, traits::{rx_tx::OnReceive, state::Initializable}};

const APP_TAG: &str = "Uart";
const APP_THREAD_NAME: &str = "uart_trd";

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

static UART_QUEUE: OnceBox<Arc<Queue>> = OnceBox::new();
const UART_QUEUE_SIZE: UBaseType = 64;

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
    pub receive: Option<ArcMux<dyn OnReceive>>,
    pub deinit: fn(&UartConfig) -> Result<()>,
}

#[derive(Clone)]
pub struct Uart<'a, const SIZE: usize = 4> {
    functions: Option<&'a UartFn>,
    config: UartConfig,
    listener: [Option<ArcMux<dyn OnReceive>>; SIZE],
    thread: Thread,
}

unsafe impl<'a> Sync for Uart<'a> {}
unsafe impl<'a> Send for Uart<'a> {}

impl OnReceive for Uart<'_> {
    fn on_receive(&self, data: &[u8]) {
        let uart_queue = UART_QUEUE.get().unwrap();
        uart_queue.post_from_isr(data).unwrap();
    }
}

impl Initializable for Uart<'_> {
    fn init(&mut self) -> Result<()> {
        if let Some(functions) = self.functions {
            if (functions.init)(&self.config).is_err() {
                return Err(Error::Unhandled("Failed to initialize Uart"))
            }


            UART_QUEUE.get_or_init(|| 
                if let Ok(queue) = Queue::new(UART_QUEUE_SIZE, 1) {
                    Box::new(Arc::new(queue))
                } else {
                    panic!("Failed to create UART queue");
                }
            );



            
            let listener_clone = self.listener.clone();
            self.thread.spawn_simple(move || {

                let uart_queue = UART_QUEUE.get().unwrap();
                
                loop {
                    let mut bytes = [0u8; UART_QUEUE_SIZE as usize];
                    uart_queue.fetch(&mut bytes, TickType::MAX).unwrap();

                    for listener in listener_clone.iter() {
                        if let Some(listener) = listener {
                            if let Ok(mut listener) = listener.lock() {
                                listener.set_source(Bytes::new_by_str("UART"));
                                listener.on_receive(&bytes);
                            }
                        }
                    }
                }

            })?;

            Ok(())
        } else {
            Err(Error::Unhandled("No functions assigned to Uart"))
        }
    }
}

impl<'a, const SIZE: usize> Uart<'a, SIZE> {
    pub fn new(config: UartConfig) -> Self {
        Self { 
            config, 
            functions: None,
            listener: [const { None }; SIZE],
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, minimal_stack_size!(), ThreadPriority::Normal),
        }
    }

    pub fn set_functions(&mut self, functions: *const UartFn) {
        unsafe {
            self.functions = Some(&(*functions));
        }
    }
    
    pub fn transmit(&self, data: &[u8]) -> usize {
        if let Some(functions) = self.functions {
            (functions.transmit)(data)
        } else {
            0
        }
    }

    pub fn add_listener(&mut self, listener: ArcMux<dyn OnReceive>) -> Result<()> {
        for slot in self.listener.iter_mut() {
            if slot.is_none() {
                *slot = Some(listener);
                return Ok(());
            }
        }
        Err(Error::Unhandled("No available slot for listener"))
    }
}