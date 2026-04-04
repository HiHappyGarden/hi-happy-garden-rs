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

use core::str::from_utf8;

use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;
use osal_rs::{access_static_option, log_error};
use osal_rs::os::{Queue, QueueFn, Thread, ThreadFn};
use osal_rs::os::types::{StackType, TickType, UBaseType};
use osal_rs::utils::{Error, Result};

use crate::apps::config::Config;
use crate::apps::session::{Session, User};
use crate::drivers::platform::ThreadPriority;
use crate::traits::rx_tx::{OnReceive, SetTransmit, Source};
use crate::traits::state::Initializable;


const APP_TAG: &str = "AppParser";
const THREAD_NAME: &str = "parser_trd";
const STACK_SIZE: StackType = 2_048;

const BUFFER_SIZE: usize = 256;
const QUEUE_SIZE: UBaseType = 64;
static mut QUEUE: Option<Queue> = None;
static mut SOURCE: Option<Source> = None;

pub(super) const CMD_SIZE : usize = 64;




pub(super) struct Parser {
    thread: Thread,
}

impl OnReceive for Parser {
    fn on_receive(&self, source: Source, data: &[u8]) -> Result<()> {
        access_static_option!(QUEUE).post(data, 0)?;

        unsafe {
            SOURCE = Some(source);
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

        Ok(())
    }
}

impl Parser {
    pub(super) fn set_transmit(&mut self, transmit: &'static dyn SetTransmit) {

        let _ = self.thread.spawn_simple(move || {
            
            let mut parser: AtParser<dyn AtContext<CMD_SIZE>, CMD_SIZE> = AtParser::new();

            let config = Config::shared();

            let commands: &mut [(&str, &mut dyn AtContext<CMD_SIZE>)] = &mut [
                (Session::AT_CMD, config.get_session()),
                (User::AT_CMD, User::get_local()),
            ];

            parser.set_commands(commands);

            let mut buffer = [0u8; BUFFER_SIZE];
            let mut buffer_count = 0;

            loop {


                let mut byte = [0u8; 1];
                if access_static_option!(QUEUE).fetch(&mut byte, TickType::MAX).is_ok() {
                    buffer[buffer_count] = byte[0];
                    buffer_count += 1;
                    if buffer_count >= buffer.len() {
                        log_error!(APP_TAG, "Buffer overflow, data too long");
                        buffer_count = 0; 
                    }
                }

                if buffer_count > 0 && buffer[buffer_count - 1] == b'\n' {
                    buffer[buffer_count - 1] = 0; 
                    buffer_count -= 1;
                    if buffer_count > 0 && buffer[buffer_count - 1] == b'\r' {
                        buffer[buffer_count - 1] = 0; 
                        buffer_count -= 1; 
                    }
                    
                    let _src = access_static_option!(SOURCE);

                    let cmd = from_utf8(&buffer[..buffer_count]).unwrap_or("<invalid utf-8>").trim();
                    
                    match parser.execute(cmd) {
                        Ok(response) => {
                            if response.is_empty() {
                                transmit.transmit(b"OK\r\n");
                            } else {
                                transmit.transmit(response.as_raw_bytes());
                                transmit.transmit(b"\r\nOK\r\n");
                            }
                        }
                        Err(_) => {
                                transmit.transmit(b"KO\r\n");
                        }
                    }

                    buffer.fill(0);
                    buffer_count = 0;
                    unsafe {
                        SOURCE = None;
                    }

                }
            }
        });
    }

    pub(super) fn shared() -> Self {
        Self {
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal),
        }
    }

}