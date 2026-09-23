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

use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Result};

use crate::apps::display::select::Select;
use crate::apps::screen_route::ScreenId;
use crate::apps::signals::display::request_redraw;
use crate::apps::sprinkler::schedule::ScheduleController;
use crate::apps::sprinkler::zone::ZoneController;
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRouteCtx, ScreenRoute, ScreenSelections, screen_selections_new};

#[derive(Copy, Clone, PartialEq, Eq)]
enum FSMState {
    Schedule,
    Zone,
}

/// Read only view of the schedules and of the zones watered by each of them.
pub(super) struct ScreenSprinkler {
    fsm_state: FSMState,
    selected_schedule: usize,
    schedule: Select<{ScheduleController::SIZE}>,
    zone: Select<{ZoneController::SIZE}>,
}

impl ScreenRoute<ScreenId> for ScreenSprinkler {

    #[inline]
    fn id(&self) -> ScreenId {
        ScreenId::Sprinkler
    }

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {
        match self.fsm_state {
            FSMState::Schedule => self.draw_schedule_state(screen_route_ctx),
            FSMState::Zone => self.draw_zone_state(screen_route_ctx),
        }
    }

}

impl ScreenSprinkler {
    pub(super) fn new() -> Self {
        Self {
            fsm_state: FSMState::Schedule,
            selected_schedule: 0,
            schedule: Select::<{ScheduleController::SIZE}>::new(),
            zone: Select::<{ZoneController::SIZE}>::new()
        }
    }

    #[inline]
    fn set_state(&mut self, display_signal: &mut EventBits, next: FSMState) {
        self.fsm_state = next;
        request_redraw(display_signal);
    }

    fn schedule_selections(&self) -> ScreenSelections<{ScheduleController::SIZE}> {
        let mut selects = screen_selections_new();

        for (i, schedule) in (&mut *ScheduleController::shared()).into_iter().enumerate().take(ScheduleController::SIZE) {
            if schedule.description.is_empty() {
                selects[i].0.format(format_args!("Schedule {}", i + 1));
            } else {
                selects[i].0 = schedule.description.clone();
            }
            selects[i].1 = i == self.selected_schedule;
        }

        selects
    }

    fn zone_selections(&self) -> ScreenSelections<{ZoneController::SIZE}> {
        let mut selects = screen_selections_new();

        if let Some(schedule) = (&mut *ScheduleController::shared()).into_iter().nth(self.selected_schedule) {
            for (select, zone) in selects.iter_mut().zip(schedule.zones.iter()) {
                match zone {
                    Some((relay, minutes)) => select.0.format(format_args!("{} {} min", relay, minutes)),
                    None => select.0.append_str("Not set"),
                }
            }
        }

        selects
    }

    fn draw_schedule_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {

        
        match self.schedule.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Schedules"),
            ScreenParam::<u16, {ScheduleController::SIZE}>::Selects(self.schedule_selections())
        )? {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(param) => {

                match param {
                    ScreenParam::Selects(selects) => self.selected_schedule = selects.iter().position(|(_, selected)| *selected).unwrap_or(0),
                    _ => self.selected_schedule = 0,
                }

                // A new schedule was picked: rebuild the zone list from scratch.
                self.zone = Select::new();
                self.set_state(display_signal, FSMState::Zone);
                Ok(Nav::Stay)
            }
            Answer::Cancelled => Ok(Nav::Pop),
        }
    }

    fn draw_zone_state(&mut self, ScreenRouteCtx { lcd, display_signal, status_signal: _, rtc}: &mut ScreenRouteCtx<'_>) -> Result<Nav<ScreenId>> {


        match self.zone.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("Zones"),
            ScreenParam::<u16, {ZoneController::SIZE}>::Selects(self.zone_selections())
        )? {
            Answer::Pending => Ok(Nav::Stay),
            // Any button goes back to the schedule list.
            Answer::Confirmed(_) | Answer::Cancelled => {
                // Rebuild the schedule list so it starts from the one just viewed.
                self.schedule = Select::new();
                self.set_state(display_signal, FSMState::Schedule);
                Ok(Nav::Stay)
            }
        }
    }

}
