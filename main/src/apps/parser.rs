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

use alloc::sync::Arc;
use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;
use osal_rs::{access_static_option, log_debug, log_error, println};
use osal_rs::os::{Mutex, MutexFn, Queue, QueueFn, Thread, ThreadFn};
use osal_rs::os::types::{StackType, TickType, UBaseType};
use osal_rs::utils::{Error, Result};

use crate::apps::session::Session;
use crate::drivers::platform::ThreadPriority;
use crate::traits::rx_tx::{OnReceive, Source};
use crate::traits::state::Initializable;


const APP_TAG: &str = "AppParser";
const THREAD_NAME: &str = "parser_trd";
const STACK_SIZE: StackType = 2_048;

const BUFFER_SIZE: usize = 256;
const QUEUE_SIZE: UBaseType = 64;
static mut QUEUE: Option<Queue> = None;
static mut SOURCE: Option<Source> = None;

pub(super) const CMD_SIZE : usize = 64;
pub(super) const RESPONSE_OK: &str = "OK";
pub(super) const RESPONSE_KO: &str = "KO";




pub(super) struct Parser {
    thread: Thread,
    session: Arc<Mutex<Option<Session>>>,
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

        let session = Arc::clone(&self.session);

        self.thread.spawn_simple(move || {
            
            let mut parser: AtParser<dyn AtContext<CMD_SIZE>, CMD_SIZE> = AtParser::new();

            let mut binding = session.lock().unwrap();
            let session = binding.as_mut().unwrap();

            let commands: &mut [(&str, &mut dyn AtContext<CMD_SIZE>)] = &mut [
                (Session::AT_CMD, session),
                //("AT+RST", &mut reset),
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

                    let cmd = from_utf8(&buffer[..buffer_count]).unwrap_or("<invalid utf-8>");

                    match parser.execute(cmd) {
                        Ok(response) => println!("Response: {}", response),  
                        Err(e) => println!("Error: {:?}", e),
                    }

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
    pub(super) fn new() -> Self {
        Self {
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal),
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub(super) fn set_session(&mut self, session: Arc<Mutex<Option<Session>>>) {
        self.session = session;
    }
}