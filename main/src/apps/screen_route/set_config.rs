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

use crate::apps::screen_route::ScreenId;
use crate::traits::screen::{Nav, ScreenRoute, ScreenRouteCtx};


pub(super) struct ScreenSetConfig;


impl ScreenRoute<'_, ScreenId> for ScreenSetConfig {
    
    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::SetConfig
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {
        
        todo!("ScreenSetConfig draw not implemented yet");

        Ok(Nav::Pop)
    }
}

impl ScreenSetConfig {
    pub(super) fn new() -> Self {
        Self
    }
}   