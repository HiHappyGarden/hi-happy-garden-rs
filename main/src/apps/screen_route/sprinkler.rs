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

#![allow(dead_code)]

use osal_rs::utils::{ Bytes, Result };

use crate::apps::display::select::Select;
use crate::apps::screen_route::ScreenId;
use crate::apps::sprinkler::schedule::ScheduleController;
use crate::apps::sprinkler::zone::ZoneController;
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::traits::screen::{ Nav, Screen, ScreenParam, ScreenRouteCtx, ScreenRoute };

static mut FSM_STATE: FSMState = FSMState::Schedule;

#[derive(Copy, Clone)]
enum FSMState {
    Schedule,
    Zone,
    End,
}

pub(super) struct ScreenSprinkler {
    schedule: Select<{ScheduleController::SIZE}>,
    zone: Select<{ZoneController::SIZE}>,
}

impl ScreenRoute<'_, ScreenId> for ScreenSprinkler {

    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::Sprinkler
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {

        let fsm_state = unsafe { *&raw const FSM_STATE };
        
        match fsm_state {
            FSMState::Schedule => self.draw_schedule_state(screen_route_ctx),
            FSMState::Zone => self.draw_zone_state(screen_route_ctx),
            FSMState::End => Ok(Nav::Stay)
        }
    }

}

impl ScreenSprinkler {
    pub fn new() -> Self {
        Self {
            schedule: Select::<{ScheduleController::SIZE}>::new(),
            zone: Select::<{ZoneController::SIZE}>::new()
        }
    }

    fn draw_schedule_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {

        

        let mut param = ScreenParam::<u16, {ScheduleController::SIZE}>::default();

        let selects = match &mut param.selects {
            Some(selects) => selects,
            None => &mut [(Bytes::<DISPLAY_INPUT_MAX_SIZE>::new(), false); ScheduleController::SIZE],
        };

        let mut count = 0;
        for schedule in &mut (*ScheduleController::shared()) {
            selects[count] = (schedule.description.clone(), false);
            count += 1;
        }

        self.schedule.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Auth"),
            param
        )?;

        todo!("draw_schedule_state is not fully implemented yet");

    }

    fn draw_zone_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav<'_, ScreenId>> {

        todo!("draw_zone_state is not implemented yet");

    }

}