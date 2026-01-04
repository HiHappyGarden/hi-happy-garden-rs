use alloc::boxed::Box;
use osal_rs::log_info;
use osal_rs::utils::Result;

/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/



use crate::drivers::platform::Hardware;
use crate::traits::button::{ButtonState, OnClickable};
use crate::traits::encoder::{EncoderDirection, OnRotatableAndClickable};
use crate::traits::hardware::HardwareFn;
use crate::traits::state::Initializable;

const APP_TAG: &str = "AppMain";

pub struct AppMain<'a> {
    hardware: &'a mut Hardware
}

impl Initializable for AppMain<'_> {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app main");


        self.hardware.get_button().set_on_click(Box::new(|state| {
            match state {
                ButtonState::Pressed => log_info!(APP_TAG, "Button Pressed"),
                ButtonState::Released => log_info!(APP_TAG, "Button Released"),
                ButtonState::None => {}
            }
        }));

        self.hardware.get_encoder().set_on_click(Box::new(|state| {
            match state {
                ButtonState::Pressed => log_info!(APP_TAG, "Encoder Pressed"),
                ButtonState::Released => log_info!(APP_TAG, "Encoder Released"),
                ButtonState::None => {}
            }
        }));

        self.hardware.get_encoder().set_on_rotate(Box::new(|direction, _position| {
            match direction {
                EncoderDirection::Clockwise => log_info!(APP_TAG, "Encoder Clockwise"),
                EncoderDirection::CounterClockwise => log_info!(APP_TAG, "Encoder CounterClockwise"),
            }

        }));

        Ok(())
    }
}

impl<'a> AppMain<'a> {
    pub fn new(hardware: &'a mut Hardware) -> Self {
        AppMain {
            hardware
        }
    }
}