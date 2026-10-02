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
use osal_rs::utils::{Bytes, Result};

use crate::apps::display::select::Select;
use crate::apps::screen_route::{BoxedScreen, ScreenId as FatherScreenId};
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRouteCtx, ScreenRoute, ScreenSelections};

use crate::apps::screen_route::schedule::ScreenSchedule;
use crate::apps::screen_route::zone::ScreenZone;
            
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(in crate::apps) enum ScreenId {
    Schedule,
    Zone
}


impl From<usize> for ScreenId {
    fn from(value: usize) -> Self {
        match value {
            0 => ScreenId::Schedule,
            1 => ScreenId::Zone,
            _ => ScreenId::Schedule,
        }
    }
}


#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct ScreenSprinkler (Select<2>);

impl ScreenRoute<FatherScreenId> for ScreenSprinkler {

    #[inline]
    fn id(&self) -> FatherScreenId {
        FatherScreenId::Sprinkler
    }

    fn draw(&mut self, ScreenRouteCtx{lcd, display_signal, rtc, ..}: &mut ScreenRouteCtx<'_>) -> Result<Nav<FatherScreenId>> {
        
        match self.0.draw(
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
                                let selected_schedule = selected.iter().position(|(_, b)| *b).unwrap_or(0);

                                // let screen: BoxedScreen = match selected_schedule.into() {
                                //     ScreenId::Schedule           => Box::new(ScreenSchedule::new()),
                                //     ScreenId::Zone               => Box::new(ScreenZone::new()),
                                // };

                                // Ok(Nav::Push(screen))
                                Ok(Nav::Stay)
                            }
                            _ => Ok(Nav::Stay)
                        }
                        
                    }
                    Answer::Cancelled => Ok(Nav::Pop),
                }
    }

}

impl ScreenSprinkler {

    #[inline]
    pub(super) fn new() -> Self {
        Self(Select::new())
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
