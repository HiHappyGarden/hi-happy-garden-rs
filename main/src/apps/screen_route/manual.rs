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

use osal_rs::println;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;

use crate::apps::display::number::Number;
use crate::apps::display::select::Select;
use crate::apps::screen_route::{StateMachine, step};
use crate::apps::sprinkler::zone::{ZoneController, ZoneRelay};
use crate::traits::screen::{Answer, Nav, Screen, ScreenParam, ScreenRoute, ScreenRouteCtx, ScreenSelections};

enum FSMState {
    Zone,
    Minutes
}


pub(super) struct ScreenManual{
    fsm_state: FSMState,
    zone: Select::<{ZoneController::SIZE}>,
    minutes: Number<u8>,
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

impl StateMachine<FSMState> for ScreenManual {
    fn next_state(&mut self, next: FSMState) {
        self.fsm_state = next;
    }
}

impl ScreenManual {
    pub(super) fn new() -> Self {

        Self {
            fsm_state: FSMState::Zone,
            zone: Select::<{ZoneController::SIZE}>::new(),
            minutes: Number::new_with_range(0..=255)
        }
    }

    fn draw_zone_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        
        let selects: ScreenSelections<{ZoneController::SIZE}> = [
            (ZoneController::shared()[0].description, false),
            (ZoneController::shared()[1].description, false),
            (ZoneController::shared()[2].description, false),
            (ZoneController::shared()[3].description, false),
        ];
        

        
        let answer = self.zone.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Zone"),
            ScreenParam::Selects(selects),
        )?;

        Ok(step(self, display_signal, answer, FSMState::Minutes, FSMState::Zone))   
    }

    fn draw_minutes_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        let answer = self.minutes.draw(
            *lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Minutes"),
            ScreenParam::Number(0),
        )?;

        if let Answer::Confirmed(ScreenParam::Number(value)) = answer {
            let zone = self.zone.get_value()?;
            let zone = zone.iter()
            .enumerate()
            .find(|(_, selected)| selected.1) 
            .map(|(index, _)| index)
            .ok_or(Error::NotFound)?;
            
            let zone = ZoneRelay::from(zone as u8);
            
            println!("No implemented action for selected zone: {:?} minutes: {:?}", zone, value);

            Ok(Nav::Pop(false))
        } else {
            Ok(step(self, display_signal, answer, FSMState::Zone, FSMState::Minutes))
        }
    }
}