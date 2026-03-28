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

use osal_rs::{access_static_option, log_debug, log_error};
use osal_rs::os::{Queue, QueueFn, Thread, ThreadFn};
use osal_rs::os::types::{StackType, TickType, UBaseType};
use osal_rs::utils::{AsSyncStr, Bytes, Error, Result};

use crate::drivers::platform::ThreadPriority;
use crate::traits::rx_tx::OnReceive;
use crate::traits::state::Initializable;


const APP_TAG: &str = "AppParser";
const THREAD_NAME: &str = "display_trd";
const STACK_SIZE: StackType = 2_560;

const QUEUE_SIZE: UBaseType = 64;
static mut QUEUE: Option<Queue> = None;
static mut ON_RECEIVE: Bytes<8> = Bytes::new();
static mut SOURCE: Option<Source> = None;

 #[derive(Debug, Clone, Copy)]
enum Source {
    Uart,
    Mqtt,
    Display
}

impl Source {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "UART" => Some(Self::Uart),
            "MQTT" => Some(Self::Mqtt),
            "DISPLAY" => Some(Self::Display),
            _ => None
        }
    }

    fn get_delimitor(_self: &Self) -> u8 {
        match _self {
            Self::Uart | Self::Display=> b'\n',
            Self::Mqtt => b'}'
        }
    }
}

impl AsSyncStr for Source {
    fn as_str(&self) -> &str {
        match self {
            Self::Uart => "UART",
            Self::Mqtt => "MQTT",
            Self::Display => "DISPLAY"
        }
    }
}

pub struct Parser {
    thread: Thread,

}

impl OnReceive for Parser {
    fn on_receive(&self, source: &str, data: &[u8]) -> Result<()> {
        access_static_option!(QUEUE).post(data, 0)?;

        unsafe {
            ON_RECEIVE = Bytes::from_str(source);
            SOURCE = Source::from_str(source);
        }

        Ok(())
    }
}

impl Initializable for Parser {
    fn init(&mut self) -> Result<()> {


        if let Ok(queue) =  Queue::new(QUEUE_SIZE, 1) {
            unsafe {
                QUEUE = Some(queue);
            }
        } else {
            log_error!(APP_TAG, "Error creating queue");
            return Err(Error::OutOfMemory)
        }

        self.thread.spawn_simple(move || {
            
        let mut buffer = [0u8; QUEUE_SIZE as usize];
        let mut buffer_count = 0;

            loop {


                let mut byte = [0u8; 1];
                if access_static_option!(QUEUE).fetch(&mut byte, TickType::MAX).is_ok() {
                    buffer[buffer_count] = byte[0];
                    buffer_count += 1;
                    
                }

                if buffer_count > 0 && buffer[buffer_count - 1] == Source::get_delimitor(access_static_option!(SOURCE)) {

                    buffer[buffer_count - 1] = 0; 
                    buffer_count -= 1;
                    if buffer[buffer_count - 1] == b'\r' {
                        buffer[buffer_count - 1] = 0; 
                        buffer_count -= 1; 
                    }
                    
                    let src = access_static_option!(SOURCE);

                    log_debug!(APP_TAG, "Received data from {:?}: {:?}", src, &buffer[..buffer_count]);

                    buffer.fill(0);
                    buffer_count = 0;
                    unsafe {
                        SOURCE = None;
                    }

                }
            }
        })?;

        Ok(())
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal)
        }
    }
}