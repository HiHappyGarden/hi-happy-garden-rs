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
use osal_rs::{access_static_option, log_error, log_info};
use osal_rs::os::{Queue, QueueFn, Thread, ThreadFn};
use osal_rs::os::types::{StackType, TickType, UBaseType};
use osal_rs::utils::{Error, Result};

use crate::apps::config::Config;
use crate::apps::session::{Session, User};
use crate::drivers::platform::ThreadPriority;
use crate::traits::rx_tx::{OnReceive, SetTransmit, Source};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::apps::signals::status::{StatusSignal, StatusFlag};


const APP_TAG: &str = "AppParser";
const THREAD_NAME: &str = "parser_trd";
const STACK_SIZE: StackType = 2_048;

const BUFFER_SIZE: usize = 256;
const QUEUE_SIZE: UBaseType = 64;

static mut QUEUE: Option<Queue> = None;
static mut SOURCE: Option<Source> = None;

static mut UART_CHANNEL: Option<&'static dyn SetTransmit> = None;

pub(super) const CMD_SIZE : usize = 64;

macro_rules! at_cmd_response {
    ($at_resp:expr; $($args:expr),+) => {
        at_parser_rs::at_response!(crate::apps::parser::CMD_SIZE, $at_resp; $($args),+)
    };
}
pub(super) use at_cmd_response;

macro_rules! clear_buffer {
    ($buffer:expr, $buffer_count:expr) => {
        $buffer.fill(0);
        $buffer_count = 0;
    };
}

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
        log_info!(APP_TAG, "Init app parser");

        if let Ok(queue) =  Queue::new(QUEUE_SIZE, 1) {
            unsafe {
                QUEUE = Some(queue);
            }
        } else {
            log_error!(APP_TAG, "Error creating queue");
            return Err(Error::OutOfMemory)
        }

        self.thread.spawn_simple(move || {
            
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

                    
                    let src = access_static_option!(SOURCE);
                    let src_status_flags = <StatusFlag as Into<u32>>::into(StatusFlag::from(src));

                    let channel = match src {
                        Source::Uart => *access_static_option!(UART_CHANNEL),
                        Source::Display | Source::Mqtt => {
                            clear_buffer!(buffer, buffer_count);
                            continue;
                        }
                    };


                    let status = StatusSignal::get();
                    let is_logged = status & <StatusFlag as Into<u32>>::into(StatusFlag::UserLogged) != 0;
                    
                    if is_logged && (status & src_status_flags == 0) {
                        channel.transmit(b"KO\r\n");
                        clear_buffer!(buffer, buffer_count);
                        continue;
                    }


                    if !is_logged {
                        StatusSignal::set(src_status_flags); // Set the status flag for the source of the command
                    } else {
                        Session::reset_timer(); // Reset the session timer on each command if logged in
                    }


                    let cmd = from_utf8(&buffer[..buffer_count]).unwrap_or("<invalid utf-8>").trim();
                    
                    

                    match parser.execute(cmd) {
                        Ok(response) => {
                            if response.is_empty() {
                                channel.transmit(b"OK\r\n");
                            } else {
                                channel.transmit(response.as_raw_bytes());
                                channel.transmit(b"\r\nOK\r\n");
                            }
                        }
                        Err(_) => {
                                channel.transmit(b"KO\r\n");
                        }
                    }

                    if !is_logged {
                        StatusSignal::clear(src_status_flags); // Clear the status flag for the source of the command
                    }
                    
                    unsafe {
                        SOURCE = None;
                    }
                    clear_buffer!(buffer, buffer_count);
                }
            }
        })?;


        Ok(())
    }
}

impl Parser {

    #[inline]
    pub(super) fn set_uart_transmit(transmit: &'static dyn SetTransmit) {
        unsafe {
            UART_CHANNEL = Some(transmit);
        }
    }

    #[allow(unused)]
    #[inline]
    pub(super) fn get_source() -> Option<Source> {
        unsafe { SOURCE }
    }

    #[inline]
    pub(super) fn shared() -> Self {
        Self {
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal),
        }
    }
}