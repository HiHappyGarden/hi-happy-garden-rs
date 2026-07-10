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

use alloc::sync::Arc;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;

use crate::apps::display::select::Select;
use crate::apps::sprinkler::schedule::Schedule;
use crate::apps::sprinkler::zone::Zone;
use crate::apps::sprinkler::Sprinkler;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{ScreenRoute};

static mut FSM_STATE: FSMState = FSMState::Schedule;

enum FSMState {
    Schedule,
    Zone,
    End,
}

pub(super) struct ScreenSprinkler {
    sprinkler: Arc<Mutex<Sprinkler>>,
    schedule: Select<{Schedule::SIZE}>,
    zone: Select<{Zone::SIZE}>,
}

impl ScreenRoute for ScreenSprinkler {
    fn draw(&mut self, 
        _lcd: &mut dyn LCDDisplayFn,
        _display_signal: &mut EventBits, 
        _status_signal: &mut EventBits, 
        _rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> osal_rs::utils::Result<()> {
        todo!("implement draw for ScreenSprinkler");
    }

    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        todo!("implement as_any_mut for ScreenSprinkler");
    }

    fn as_any(&self) -> &dyn core::any::Any {
        todo!("implement as_any for ScreenSprinkler");
    }
}

impl ScreenSprinkler {
    pub fn new(sprinkler: Arc<Mutex<Sprinkler>>) -> Self {
        
        //let mut schedule = screen_selections_new::<4usize>::();
        
        
        
        let mut ret = Self {
            sprinkler,
            schedule: Select::new(),
            zone: Select::new()
        };



        ret
    }
    
}