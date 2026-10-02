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

use osal_rs::utils::Result;

use crate::apps::display::text::Text;
use crate::apps::screen_route::sprinkler::ScreenId;
use crate::traits::screen::{Nav, ScreenRoute, ScreenRouteCtx};

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ScreenZone {
    foo: Text,
}

impl ScreenRoute<ScreenId> for ScreenZone {

    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::Zone
    }

    fn draw(&mut self, ScreenRouteCtx{lcd: _, display_signal: _, rtc: _, ..}: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {

        //todo!("ScreenZone::draw not implemented yet");
        
        Ok(Nav::Stay)
    }

}

impl ScreenZone {
    pub(super) fn new() -> Self {
        Self {
            foo: Text::new(),
        }
    }


}