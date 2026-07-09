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

use core::any::Any;
use alloc::sync::Arc;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{AsSyncStr, Bytes, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::drivers::date_time::DateTime;
use crate::traits::integer::Integer;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;

pub type ScreenCallback<N = u16, const N_SELECTS: usize = 6> = Option<fn(Option<ScreenParam<N, N_SELECTS>>, confirmed: bool)>;
pub type ScreenSelections<const N: usize = 6> = [(Bytes<{DISPLAY_INPUT_MAX_SIZE}>, bool); N];

pub const fn screen_selections_new<const N: usize>() -> ScreenSelections<N> {
    [(Bytes::new(), false); N]
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct ScreenParam<N = u16, const N_SELECTS: usize = 6> 
where N: Integer
{
    pub check: Option<bool>,
    pub input: Option<Bytes<{DISPLAY_INPUT_MAX_SIZE}>>,
    pub input_secret_mode: Option<bool>,
    pub number: Option<N>,
    pub date_time: Option<DateTime>,
    pub selects: Option<ScreenSelections<N_SELECTS>>,
}


impl<N, const N_SELECTS: usize> Default for ScreenParam<N, N_SELECTS>
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


pub trait Screen<T, N = u16, const N_SELECTS: usize = 6>
where N: Integer
{
     fn draw(&mut self,
        lcd: &mut dyn LCDDisplayFn,
        signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
        text: &dyn AsSyncStr,
        param: ScreenParam<N, N_SELECTS>,
        callback: ScreenCallback<N, N_SELECTS>
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
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> Result<()>;

    #[allow(unused)]
    fn as_any_mut(&mut self) -> &mut dyn Any;

    #[allow(unused)]
    fn as_any(&self) -> &dyn Any;
}
