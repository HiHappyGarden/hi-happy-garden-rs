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

#[cfg(feature = "tests")]
pub(super) mod tests;

use core::str::from_utf8;
use core::sync::atomic::{AtomicU8, Ordering};
use core::time::Duration;

use at_parser_rs::AtError;
use at_parser_rs::context::AtContext;
use at_parser_rs::parser::AtParser;
use osal_rs::{log_error, log_info};
use osal_rs::os::{Queue, QueueFn, Thread, ThreadFn};
use osal_rs::os::types::{EventBits, StackType, TickType, UBaseType};
use osal_rs::utils::{Error, OnceBox, Result};

use crate::apps::config::{Config, ConfigAt, DaylightSavingTime, WifiConfig, NtpConfig};
use crate::apps::session::{Session, User};
use crate::apps::sprinkler::schedule::ScheduleController;
use crate::apps::sprinkler::zone::ZoneController;
use crate::apps::system_handler::SystemHandler;
use crate::drivers::platform::ThreadPriority;
use crate::traits::rx_tx::{OnReceive, SetTransmit, Source};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;
use crate::apps::signals::status::{StatusSignal, StatusFlag};


const APP_TAG: &str = "AppParser";

const THREAD_NAME: &str = "app_parser_thr";
const STACK_SIZE: StackType = 1_024 * 4; // words: factory reset (remove_recursive) and config saves need > 8KB
const NEW_LINE: &str = "\r\n";
const OK_RESPONSE: &str = "OK";
const KO_RESPONSE: &str = "KO";

const BUFFER_SIZE: usize = 256;
const QUEUE_SIZE: UBaseType = BUFFER_SIZE as UBaseType;

static QUEUE: OnceBox<Queue> = OnceBox::new();
/// Source of the line being received, written from the UART interrupt:
/// `SOURCE_NONE` or a [`source_to_u8`] value
static SOURCE: AtomicU8 = AtomicU8::new(SOURCE_NONE);
const SOURCE_NONE: u8 = 0;

// Set once at startup, read by the parser thread
static UART_CHANNEL: OnceBox<&'static dyn SetTransmit> = OnceBox::new();
static MQTT_CHANNEL: OnceBox<&'static dyn SetTransmit> = OnceBox::new();

#[inline]
const fn source_to_u8(source: Source) -> u8 {
    match source {
        Source::Uart => 1,
        Source::Mqtt => 2,
    }
}

#[inline]
const fn source_from_u8(value: u8) -> Option<Source> {
    match value {
        1 => Some(Source::Uart),
        2 => Some(Source::Mqtt),
        _ => None,
    }
}

macro_rules! at_cmd_response {
    ($at_resp:expr; $($args:expr),+) => {
        at_parser_rs::at_response!(crate::apps::parser::Parser::CMD_SIZE, $at_resp; $($args),+)
    };
}
pub(super) use at_cmd_response;

macro_rules! clear_buffer {
    ($buffer:expr, $buffer_count:expr) => {
        $buffer.fill(0);
        $buffer_count = 0;
    };
}

pub(super) struct Parser (Thread);

impl OnReceive for Parser {

    fn on_receive(&self, source: Source, data: &[u8]) -> Result<()> {
        
        let local_source = source_to_u8(source);

        // A line already started from the other source keeps it
        match SOURCE.compare_exchange(SOURCE_NONE, local_source, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => {}
            Err(current) if current == local_source => {}
            Err(_) => return Err(Error::InvalidType),
        }

        let queue = QUEUE.get().ok_or(Error::Unhandled("Queue not initialized"))?;

        for &byte in data {
            match &source {
                Source::Uart => queue.post_from_isr(&[byte])?,
                Source::Mqtt => queue.post_with_to_tick(&[byte], Duration::from_millis(100))?,
            }
        }

        Ok(())
    }
}

impl Initializable for Parser {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app parser");

        QUEUE.get_or_try_init(|| Queue::new(QUEUE_SIZE, 1))?;

        self.0 = self.0.spawn_simple(move || {

            let mut parser: AtParser<dyn AtContext<{Parser::CMD_SIZE}>, {Parser::CMD_SIZE}> = AtParser::new();


            // Forwarders, not `&mut` into `Config`: they would all alias the same static
            let mut config_at = ConfigAt::<Config>(|config| config);
            let mut session_at = ConfigAt::<Session>(Config::get_session);
            let mut daylight_saving_time_at = ConfigAt::<DaylightSavingTime>(Config::get_daylight_saving_time);
            let mut wifi_config_at = ConfigAt::<WifiConfig>(Config::get_wifi_config);
            let mut ntp_config_at = ConfigAt::<NtpConfig>(Config::get_ntp_config_mut);

            let commands: &mut [(&str, &str, &mut dyn AtContext<{Parser::CMD_SIZE}>)] = &mut [
                (Config::AT_CMD, Config::AT_RESP, &mut config_at),
                (Session::AT_CMD, Session::AT_RESP, &mut session_at),
                (User::AT_CMD, User::AT_RESP, User::get_local()),
                (SystemHandler::AT_CMD, SystemHandler::AT_RESP, SystemHandler::get()),
                (DaylightSavingTime::AT_CMD, DaylightSavingTime::AT_RESP, &mut daylight_saving_time_at),
                (WifiConfig::AT_CMD, WifiConfig::AT_RESP, &mut wifi_config_at),
                (NtpConfig::AT_CMD, NtpConfig::AT_RESP, &mut ntp_config_at),
                (ScheduleController::AT_CMD, ScheduleController::AT_RESP, ScheduleController::shared()),
                (ZoneController::AT_CMD, ZoneController::AT_RESP, ZoneController::shared()),
            ];

            parser.set_commands(commands);

            let mut buffer = [0u8; BUFFER_SIZE];
            let mut buffer_count = 0;

            let queue = QUEUE.get().ok_or(Error::Unhandled("Queue not initialized"))?;

            loop {


                let mut byte = [0u8; 1];
                if queue.fetch(&mut byte, TickType::MAX).is_ok() {
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

                    let Some(src) = source_from_u8(SOURCE.swap(SOURCE_NONE, Ordering::AcqRel)) else {
                        log_error!(APP_TAG, "Source is not set");
                        clear_buffer!(buffer, buffer_count);
                        continue;
                    };
                    let source_flag: EventBits = <StatusFlag as Into<EventBits>>::into(StatusFlag::from(&src));

                    let channel = match src {
                        Source::Uart => UART_CHANNEL.get().copied(),
                        Source::Mqtt => MQTT_CHANNEL.get().copied(),
                    };
                    let Some(channel) = channel else {
                        log_error!(APP_TAG, "Channel is not set");
                        clear_buffer!(buffer, buffer_count);
                        continue;
                    };


                    let status = StatusSignal::get();
                    let mut is_logged = status & <StatusFlag as Into<EventBits>>::into(StatusFlag::UserLogged) != 0;
                    
                    if is_logged && (status & source_flag == 0) {
                        channel.transmit(KO_RESPONSE.as_bytes());
                        channel.transmit(NEW_LINE.as_bytes());
                        clear_buffer!(buffer, buffer_count);
                        continue;
                    }

                    let cmd = from_utf8(&buffer[..buffer_count]).unwrap_or("<invalid utf-8>").trim();
                    
                    

                    match parser.execute(cmd) {
                        Ok((at_response, response)) => {
                            let status = StatusSignal::get();
                            let new_is_logged = status & <StatusFlag as Into<EventBits>>::into(StatusFlag::UserLogged) != 0;
                            if !is_logged && new_is_logged {
                                StatusSignal::set(source_flag);
                                is_logged = true;
                            }

                            if response.is_empty() {
                                channel.transmit(at_response.as_bytes());
                                channel.transmit(OK_RESPONSE.as_bytes());
                                channel.transmit(NEW_LINE.as_bytes());
                            } else {
                                channel.transmit(at_response.as_bytes());
                                channel.transmit(response.as_raw_bytes());
                                channel.transmit(NEW_LINE.as_bytes());
                            }

                            if is_logged {
                                Session::reset_timer(); // Reset the session timer on each command if logged in
                            }    
                        }
                        Err((at_response, AtError::Unhandled(error)))  => {
                            if error.is_empty() {
                                channel.transmit(at_response.as_bytes());
                                channel.transmit(OK_RESPONSE.as_bytes());
                                channel.transmit(NEW_LINE.as_bytes());
                            } else {
                                channel.transmit(at_response.as_bytes());
                                channel.transmit(error.as_bytes());
                                channel.transmit(NEW_LINE.as_bytes());
                            }

                            if is_logged {
                                Session::reset_timer(); // Reset the session timer on each command if logged in
                            }
                        }
                        Err((at_response, AtError::UnhandledOwned(error)))  => {
                            if error.is_empty() {
                                channel.transmit(at_response.as_bytes());
                                channel.transmit(KO_RESPONSE.as_bytes());
                                channel.transmit(NEW_LINE.as_bytes());
                            } else {
                                channel.transmit(at_response.as_bytes());
                                channel.transmit(error.as_bytes());
                                channel.transmit(NEW_LINE.as_bytes());
                            }


                            if is_logged {
                                Session::reset_timer(); // Reset the session timer on each command if logged in
                            }
                        }
                        Err((at_response, _)) => {
                            channel.transmit(at_response.as_bytes());
                            channel.transmit(KO_RESPONSE.as_bytes());
                            channel.transmit(NEW_LINE.as_bytes());
                        }
                    }

                    if !is_logged {
                        StatusSignal::clear(source_flag); // Clear the status flag for the source of the command
                    }

                    clear_buffer!(buffer, buffer_count);
                }
            }
        })?;


        Ok(())
    }
}

impl Parser {
    pub(super) const NOT_LOGGED_RESPONSE: &str = "KO: Not logged in";
    pub(super) const CMD_SIZE : usize = 96;


    #[inline]
    /// Only the first channel set is kept
    pub(super) fn set_uart_transmit(transmit: &'static dyn SetTransmit) {
        let _ = UART_CHANNEL.get_or_try_init(|| Ok::<_, Error>(transmit));
    }


    #[allow(unused)]
    #[inline]
    pub(super) fn get_source() -> Option<Source> {
        source_from_u8(SOURCE.load(Ordering::Acquire))
    }

    #[inline]
    pub(super) fn shared() -> Self {
        Self (Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal))
    }

}