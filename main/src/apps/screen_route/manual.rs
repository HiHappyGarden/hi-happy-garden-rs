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

use crate::apps::DISPLAY_INPUT_MAX_SIZE;

use crate::apps::display::number::Number;
use crate::apps::display::select::Select;
use crate::apps::sprinkler::zone::ZoneController;
use crate::traits::screen::{Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx};

enum FSMState {
    Zone,
    Minutes
}


pub(super) struct ScreenManual{
    fsm_state: FSMState,
    zone: Select::<{ZoneController::SIZE}>,
    minutes: Number<u8>,
}

impl ScreenManual {
    pub(super) fn new() -> Self {
        Self {
            fsm_state: FSMState::Zone,
            zone: Select::<{ZoneController::SIZE}>::new(),
            minutes: Number::new_with_range(0..=255)
        }
    }
}

impl ScreenRoute for ScreenManual {
    fn id() -> &'static str {
        "ScreenManual"
    }

    fn renderize(&mut self, _screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav> {

        match self.fsm_state {
            FSMState::Zone => self.draw_zone_state(_screen_route_ctx)?,
            FSMState::Minutes => self.draw_minutes_state(_screen_route_ctx)?,
        };

        Ok(Nav::Stay)
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

impl ScreenManual {

    fn draw_zone_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        let answer = self.zone.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Zone"),
            ScreenParam::Selects(ZoneController::new_selections()),
        )?;

        //Ok(self.step(display_signal, answer, FSMState::Minutes, FSMState::Zone))
        Ok(Nav::Stay)
    }

    fn draw_minutes_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        // let answer = self.minutes.draw(
        //     *lcd,
        //     display_signal,
        //     rtc,
        //     &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Minutes"),
        //     ScreenParam::Input { value: self.minutes.get(), secret_mode: false },
        // )?;

        // Ok(self.step(display_signal, answer, FSMState::Zone, FSMState::Minutes))
        Ok(Nav::Stay)
    }
}