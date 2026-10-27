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

use crate::traits::screen::{ScreenRoute, ScreenRouteCtx, Nav};

pub(super) struct ScreenManual;

impl ScreenManual {
    pub(super) fn new() -> Self {
        Self
    }
}

impl ScreenRoute for ScreenManual {
    fn id() -> &'static str {
        "ScreenManual"
    }

    fn renderize(&mut self, _screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav> {
        Ok(Nav::Stay)
    }

    fn requires_auth(&self) -> bool {
        false
    }
}