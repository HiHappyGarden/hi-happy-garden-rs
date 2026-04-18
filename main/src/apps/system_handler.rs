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

use at_parser_rs::{AtError, AtResult};
use at_parser_rs::context::AtContext;

use crate::apps::parser::{CMD_SIZE, NOT_LOGGED_RESPONSE, at_cmd_response};
use crate::apps::signals::error::ErrorSignal;
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::drivers::error::HardwareErrorSignal;
use crate::drivers::filesystem::Filesystem;
use crate::drivers::platform::Hardware;
use crate::traits::signal::Signal;


static mut SYSTEM_HANDLER: SystemHandler = SystemHandler;

pub struct SystemHandler;
    
impl AtContext<{CMD_SIZE}> for SystemHandler {

    #[inline]
    fn query(&mut self, at_response: &'static str) -> AtResult<'_, { CMD_SIZE }> {
        Ok(at_cmd_response!(at_response; HardwareErrorSignal::get(), ErrorSignal::get(), StatusSignal::get()))
    }

    #[inline]
    /// rb = reboot, fr = factory reset, hwe = hardware error, e = error, s = status,
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, {CMD_SIZE}> {
        Ok(at_cmd_response!(at_response; "<rs|fr|hwe|e|s>"))
    }

    fn set(&mut self, at_response: &'static str, args: at_parser_rs::Args) -> AtResult<'_, { CMD_SIZE }> {
        if StatusSignal::get() & <StatusFlag as Into<u32>>::into(StatusFlag::UserLogged) == 0 {
            return Err((at_response, AtError::Unhandled(NOT_LOGGED_RESPONSE)));
        }
        let cmd = args.get(0).ok_or((at_response, AtError::InvalidArgs))?;
        match cmd.as_ref() {
            "rs" => 
                // Reset the system
                Hardware::reset(),
            
            "fr" => {
                // Factory reset the system
                Filesystem::remove_recursive("/").map_err(|_| (at_response, AtError::Unhandled("Failed to remove filesystem")))?;
                Hardware::reset();
            }
            _ => Err((at_response, AtError::InvalidArgs))
        }
    }
}

impl SystemHandler {
    pub const AT_CMD: &'static str = "AT+SYS";
    pub const AT_RESP: &'static str = "+SYS: ";

    pub fn get() -> &'static mut SystemHandler {
        unsafe { &mut *&raw mut SYSTEM_HANDLER }
    }
}