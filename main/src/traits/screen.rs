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
use osal_rs::utils::{AsSyncStr, Bytes, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::drivers::date_time::DateTime;
use crate::traits::integer::Integer;
use crate::traits::lcd_display::LCDDisplayFn;

pub type ScreenCallback<N = u16> = Option<fn(Option<ScreenParam<N>>, confirmed: bool)>;
pub type ScreenSelections = [Bytes<{DISPLAY_INPUT_MAX_SIZE}>; 6];

pub const fn screen_selections_new() -> ScreenSelections {
    [
        Bytes::new(),
        Bytes::new(),
        Bytes::new(),
        Bytes::new(),
        Bytes::new(),
        Bytes::new(),
    ]
}

#[derive(Debug, Clone)]
pub struct ScreenParam<N = u16> 
where N: Integer
{
    pub check: Option<bool>,
    pub input: Option<Bytes<{DISPLAY_INPUT_MAX_SIZE}>>,
    pub input_secret_mode: Option<bool>,
    pub number: Option<N>,
    pub date_time: Option<DateTime>,
    pub selects: Option<ScreenSelections>,
}


impl<N> Default for ScreenParam<N>
where N: Integer
{
    fn default() -> Self {
        Self {
            check: None,
            input: None,
            input_secret_mode: None,
            number: None,
            date_time: None,
            selects: None,
        }
    }
}


pub trait Screen<T, N = u16>
where N: Integer
{
     fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        signal: &mut EventBits, 
        date_time: &DateTime, 
        text: &dyn AsSyncStr, 
        param: ScreenParam<N>, 
        callback: ScreenCallback<N>
    ) -> Result<()>;

    fn get_value(&self) -> Result<T>;
}

pub trait ScreenRoute<N = u16>
where N: Integer
{
     fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits, 
        status_signal: &mut EventBits, 
        date_time: &DateTime
    ) -> Result<()>;
}
