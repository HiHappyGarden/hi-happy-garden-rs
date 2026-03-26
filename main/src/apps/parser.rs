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

use osal_rs::{log_debug, utils::Result};

use crate::traits::{rx_tx::OnReceive, state::Initializable};


const APP_TAG: &str = "AppParser";

pub struct Parser;

impl OnReceive for Parser {
    fn on_receive(&self, source: &str, data: &[u8]) {
        log_debug!(APP_TAG, "Received data from source: {}, data: {:02X?}", source, data);
    }
}

impl Initializable for Parser {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Parser {
    pub const fn new() -> Self {
        Self
    }
}