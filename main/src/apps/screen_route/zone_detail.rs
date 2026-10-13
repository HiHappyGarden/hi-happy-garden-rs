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

#[cfg(feature = "tests")]
pub(super) mod tests;

use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::display::input::Input;
use crate::apps::display::number::Number;
use crate::apps::display::select::Select;
use crate::apps::signals::display::request_redraw;
use crate::apps::sprinkler::commons::Status;
use crate::traits::integer::Integer;
use crate::traits::screen::{Answer, Screen, ScreenParam};
use crate::apps::sprinkler::zone::{Zone, ZoneController, ZoneRelay};
use crate::traits::screen::{Nav, ScreenRoute, ScreenRouteCtx};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FSMState {
    Description,
    Weight,
    Status
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ScreenZoneDetail {
    zone_relay: usize,
    fsm_state: FSMState,
    description: Input,
    weight: Number<u8>, 
    status: Select<2>
}

impl ScreenRoute for ScreenZoneDetail {
    fn id() -> &'static str {
        "ScreenZoneDetail"
    }

    fn renderize(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav> {


        match self.fsm_state {
            FSMState::Description => self.draw_description_state(screen_route_ctx),
            FSMState::Weight      => self.draw_weight_state(screen_route_ctx),
            FSMState::Status      => self.draw_status_state(screen_route_ctx),
        }
        
    }

}

impl ScreenZoneDetail {
    pub(super) fn new(zone_relay: usize) -> Result<Self> {
        if zone_relay >= ZoneController::SIZE {
            return Err(Error::OutOfIndex)
        }
        Ok(Self {
            zone_relay,
            fsm_state: FSMState::Description,
            description: Input::new(),
            weight: Number::new_with_range(0..9),
            status: Select::new(),
        })
    }


    fn draw_description_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {

        
        let zone_relay = ZoneRelay::from(self.zone_relay as u8);

        let answer = self.description.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(zone_relay.as_str()),
                ScreenParam::Input { 
                    value: ZoneController::shared()[self.zone_relay].description,
                    secret_mode: false 
                }
            )?;

        Ok(self.step(display_signal, answer, FSMState::Weight, FSMState::Description))
    }

    fn draw_weight_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        
        let answer = self.weight.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Weight"),
                ScreenParam::Number(0)
            )?;

        Ok(self.step(display_signal, answer, FSMState::Status, FSMState::Description))
    }

    fn draw_status_state(&mut self, ScreenRouteCtx{ lcd, display_signal, rtc, ..} :&mut ScreenRouteCtx<'_>) -> Result<Nav> {
        
        let selects: [(Bytes<{DISPLAY_INPUT_MAX_SIZE}>, bool); 2] = [
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(Status::ACTIVE.as_str()), true),
            (Bytes::<DISPLAY_INPUT_MAX_SIZE>::from(Status::UNACTIVE.as_str()), false),
        ];
        
        let answer = self.status.draw(
                *lcd,
                display_signal,
                rtc,
                &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from("Status"),
                ScreenParam::Selects(selects)
            )?;

        match answer {
            Answer::Pending => Ok(Nav::Stay),
            Answer::Confirmed(answer) => {
                match answer {
                    ScreenParam::Selects(selected) => {

                        let status  = selected
                            .iter()
                            .find(|(_, selected)| *selected)
                            .ok_or(Error::OutOfIndex)?
                            .0
                            .clone();

                        let new_zone = Zone {
                            description: self.description.get_value()?,
                            weight: self.weight.get_value().unwrap_or(0),
                            zone_relay: ZoneRelay::from(self.zone_relay as u8),
                            status: Status::from(status.as_str()),
                        };
                        ZoneController::shared()[self.zone_relay] = new_zone;

                        ZoneController::shared().save()?;

                        Ok(Nav::Pop(true))
                    }
                    _ => Err(Error::InvalidType),
                }
            }
            Answer::Cancelled => {
                self.set_state(display_signal, FSMState::Weight);
                Ok(Nav::Stay)
            }
        }
    }

    #[inline]
    fn set_state(&mut self, display_signal: &mut EventBits, next: FSMState) {
        self.fsm_state = next;
        request_redraw(display_signal);
    }


    fn step<N, const N_SELECTS: usize>(&mut self, display_signal: &mut EventBits, answer: Answer<N, N_SELECTS>, confirmed: FSMState, cancelled: FSMState) -> Nav
    where N: Integer
    {
        match answer {
            Answer::Pending => {}
            Answer::Confirmed(_) => self.set_state(display_signal, confirmed),
            Answer::Cancelled => self.set_state(display_signal, cancelled),
        }
        Nav::Stay
    }


}