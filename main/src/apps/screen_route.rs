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

use crate::apps::display::number::Number;
use crate::apps::display::input::Input;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{AsSyncStr, Result};

use crate::drivers::date_time::DateTime;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::screen::{Screen, ScreenCallback, ScreenParam};

#[derive(Clone, Copy)]
pub enum RouteConfig {
    Check,
    Date,
    Input,
    Number { min: u8, max: u8 },
    Text,
    Time,
}

impl RouteConfig {
    pub const fn create(self) -> RouteScreen {
        match self {
            Self::Input => RouteScreen::Input(Input::new()),
            Self::Number { min, max } => RouteScreen::Number(Number::new(min, max)),
        }
    }
}

pub enum RouteScreen {
    Input(Input),
    Number(Number<u8>),
}

impl Screen for RouteScreen {
    fn draw(
        &mut self,
        lcd: &mut impl LCDDisplayFn,
        signals: &mut EventBits,
        date_time: &DateTime,
        text: &impl AsSyncStr,
        param: ScreenParam,
        callback: ScreenCallback,
    ) -> Result<()> {
        match self {
            Self::Input(screen) => screen.draw(lcd, signals, date_time, text, param, callback),
            Self::Number(screen) => screen.draw(lcd, signals, date_time, text, param, callback),
        }
    }
}

pub const ROUTE_CONFIG: [RouteConfig; 2] = [
    RouteConfig::Input,
    RouteConfig::Number { min: 0, max: 100 },
];