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
use osal_rs::os::{Queue, QueueFn};
use osal_rs::os::types::UBaseType;
use osal_rs::utils::{Bytes, Error, Result};

use crate::traits::rx_tx::OnReceive;
use crate::traits::state::Initializable;


const APP_TAG: &str = "AppParser";
const QUEUE_SIZE: UBaseType = 64;
static mut QUEUE: Option<Queue> = None;
static mut ON_RECEIVE: Bytes<8> = Bytes::new();

pub struct Parser;

impl OnReceive for Parser {
    fn on_receive(&self, source: &str, data: &[u8]) -> Result<()> {
        access_static_option!(QUEUE).post(data, 0)?;

        unsafe {
            ON_RECEIVE = Bytes::from_str(source);
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
    pub const fn new() -> Self {
        Self
    }
}