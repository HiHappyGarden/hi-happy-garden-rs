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
use crate::apps::screen_route::ScreenId;
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRouteCtx, ScreenRoute, ScreenSelections};


impl From<usize> for ScreenId {
    fn from(value: usize) -> Self {
        match value {
            0 => ScreenId::Schedule,
            1 => ScreenId::Zone,
            _ => ScreenId::Sprinkler,
        }
    }
}


#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct ScreenSprinkler {
    selected_schedule: usize,
    selects: Select<2>,
}

impl ScreenRoute<ScreenId> for ScreenSprinkler {

    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::Sprinkler
    }

    fn draw(&mut self, ScreenRouteCtx{lcd, display_signal, rtc, ..}: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        
        match self.selects.draw(
                    *lcd,
                    display_signal,
                    rtc,
                    &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Select an option"),
                    ScreenParam::Selects(self.select_selections())
                )? {
                    Answer::Pending => Ok(Nav::Stay),
                    Answer::Confirmed(param) => {
                        match param {
                            ScreenParam::Selects(selected) => {
                                self.selected_schedule = selected.iter().position(|(_, b)| *b).unwrap_or(0);

                                Ok(Nav::PushId(self.selected_schedule.into()))
                                
                            }
                            _ => Ok(Nav::Stay)
                        }
                        
                    }
                    Answer::Cancelled => Ok(Nav::Pop),
                }
    }

}

impl ScreenSprinkler {
    pub(super) fn new() -> Self {
        Self {
            selected_schedule: 0,
            selects: Select::new(),
        }
    }


    #[inline]
    fn select_selections(&self) -> ScreenSelections<2> {
        let selects: [(Bytes<_>, bool); 2] = [
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Schedule"), false),
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Zone"), false),
        ];

        ScreenSelections::from(selects)
    }

}
