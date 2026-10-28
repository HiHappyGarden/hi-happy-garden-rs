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

use alloc::boxed::Box;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::display::select::Select;
use crate::apps::screen_route::schedule_detail::ScreenScheduleDetail;
use crate::apps::sprinkler::schedule::ScheduleController;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};
use crate::apps::DISPLAY_INPUT_MAX_SIZE;

#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct ScreenSchedule (Select<{ScheduleController::SIZE}>);

impl ScreenRoute for ScreenSchedule {
    fn id() -> &'static str {
        "ScreenSchedule"
    }

    fn renderize(&mut self, ScreenRouteCtx{lcd, display_signal, rtc, ..}: &mut ScreenRouteCtx<'_>) -> Result<Nav> {

        match self.0.draw(
                    *lcd,
                    display_signal,
                    rtc,
                    &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Select an option"),
                    ScreenParam::Selects(ScheduleController::new_selections())
                )? {
                    Answer::Pending => Ok(Nav::Stay),
                    Answer::Confirmed(param) => {
                        match param {
                            ScreenParam::Selects(selected) => {
                                let index = selected
                                    .iter()
                                    .position(|(_, selected)| *selected)
                                    .ok_or(Error::Empty)?;
                            
                                let screen = Box::new(ScreenScheduleDetail::new(index)?);

                                Ok(Nav::Push{
                                    id: ScreenScheduleDetail::id(), 
                                    screen
                                })
                                
                            }
                            _ => Ok(Nav::Pop(false)),
                        }
                    }
                    Answer::Cancelled => Ok(Nav::Pop(false)),
                }
    }
    
    #[inline]
    fn reinit(&mut self) { 
        self.0.clear();
    }

}

impl ScreenSchedule {
    pub(super) fn new() -> Self {
        Self(
            Select::<{ScheduleController::SIZE}>::new(),
        )
    }


}