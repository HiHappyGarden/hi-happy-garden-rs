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

use alloc::boxed::Box;
use alloc::sync::Arc;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{AsSyncStr, Bytes, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::drivers::date_time::DateTime;
use crate::traits::integer::Integer;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;

pub type ScreenSelections<const N_SELECTS: usize = 6> = [(Bytes<{DISPLAY_INPUT_MAX_SIZE}>, bool); N_SELECTS];


pub struct ScreenRouteCtx<'a> {
    pub lcd: &'a mut dyn LCDDisplayFn,
    pub display_signal: &'a mut EventBits,
    pub status_signal: &'a mut EventBits,
    pub rtc: &'a Arc<Mutex<dyn RTC + 'static>>,
}

pub enum Nav<Id> {
    Stay,
    Push(Box<dyn ScreenRoute<Id>>),
    /// Like [`Nav::Push`] but the screen is built by the router, so a screen
    /// does not need to depend on the screens it can navigate to.
    PushId(Id),
    Pop,
    PopTo(Id),
    Replace(Box<dyn ScreenRoute<Id>>),
}

pub enum Answer<N = u16, const N_SELECTS: usize = 6>
where N: Integer
{
    Pending,                            // widget during user input in editing
    Confirmed(ScreenParam<N, N_SELECTS>),
    Cancelled,
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
        param: ScreenParam<N, N_SELECTS>
    ) -> Result<Answer<N, N_SELECTS>>;

    fn get_value(&self) -> Result<T>;
}

pub trait ScreenRoute<Id>
where Id: Copy + PartialEq
{
    fn id(&self) -> Id;

    fn draw(&mut self, screen_route_ctx: &mut ScreenRouteCtx<'_>) -> Result<Nav<Id>>;

    fn requires_auth(&self) -> bool { true }
}

pub const fn screen_selections_new<const N_SELECTS: usize>() -> ScreenSelections<N_SELECTS> {
    [(Bytes::new(), false); N_SELECTS]
}
