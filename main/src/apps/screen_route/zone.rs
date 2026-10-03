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

use osal_rs::utils::{Bytes, Result};

use crate::apps::display::select::Select;
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::traits::screen::{Answer, Screen, ScreenParam};
use crate::apps::sprinkler::zone::ZoneController;
use crate::traits::screen::{Nav, ScreenRoute, ScreenRouteCtx};

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ScreenZone (Select<{ZoneController::SIZE}>);

impl ScreenRoute for ScreenZone {
    fn id() -> &'static str {
        "ScreenZone"
    }

    fn renderize(&mut self, ScreenRouteCtx{lcd, display_signal, rtc, ..}: &mut ScreenRouteCtx<'_>) -> Result<Nav> {

            match self.0.draw(
                    *lcd,
                    display_signal,
                    rtc,
                    &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Select an option"),
                    ScreenParam::Selects(ZoneController::new_selections())
                )? {
                    Answer::Pending => Ok(Nav::Stay),
                    Answer::Confirmed(_param) => {
                        // match param {
                        //     ScreenParam::Selects(selected) => {

                        //         Ok(Nav::Push{id:screen.0, screen:screen.1})
                        //     }
                        //     _ => Ok(Nav::Stay)
                        // }

                        todo!("Handle confirmed selection for ScreenZone")
                    }
                    Answer::Cancelled => Ok(Nav::Pop),
                }
        
    }

}

impl ScreenZone {
    pub(super) fn new() -> Self {
        Self(Select::new())
    }

}