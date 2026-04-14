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

use at_parser_rs::AtResult;
use at_parser_rs::context::AtContext;

use crate::apps::parser::{CMD_SIZE, at_cmd_response};

static mut SYSTEM_HANDLER: SystemHandler = SystemHandler;

pub struct SystemHandler;




impl AtContext<{CMD_SIZE}> for SystemHandler {

    #[inline]
    /// sv = save, ld = load, rb = reboot, fr = factory reset
    fn test(&mut self, at_response: &'static str) -> AtResult<'_, {CMD_SIZE}> {
        Ok(at_cmd_response!(at_response; "sv|ld|rb|fr"))
    }

}

impl SystemHandler {
    pub const AT_CMD: &'static str = "AT+SYS";
    pub const AT_RESP: &'static str = "+SYS: ";

    pub fn get() -> &'static mut SystemHandler {
        unsafe { &mut *&raw mut SYSTEM_HANDLER }
    }
}