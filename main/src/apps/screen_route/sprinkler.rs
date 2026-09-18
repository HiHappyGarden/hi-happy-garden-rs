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
use osal_rs::utils::Bytes;
use osal_rs::utils::Error;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use crate::apps::display::select::Select;
use crate::apps::signals::display::DisplayFlag;
use crate::apps::sprinkler::schedule::ScheduleController;
use crate::apps::sprinkler::zone::ZoneController;
use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::Screen;
use crate::traits::screen::ScreenParam;
use crate::traits::screen::{ScreenRoute};

static mut FSM_STATE: FSMState = FSMState::Schedule;
static UPDATE_DRAW: AtomicBool = AtomicBool::new(false);

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

impl ScreenRoute for ScreenSprinkler {
    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits, 
        status_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> osal_rs::utils::Result<()> {
        Self::apply_pending_draw(display_signal);
        Self::apply_pending_draw(display_signal);

        let fsm_state = unsafe { *&raw const FSM_STATE };
        
        match fsm_state {
            FSMState::Schedule => self.draw_schedule_state(lcd, display_signal, rtc)?,
            FSMState::Zone => self.draw_zone_state(lcd, display_signal, rtc)?,
            FSMState::End => return Ok(())
        }

        Err(Error::ReturnWithCode(1))
    }

    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        todo!("implement as_any_mut for ScreenSprinkler");
    }

    fn as_any(&self) -> &dyn core::any::Any {
        todo!("implement as_any for ScreenSprinkler");
    }
}

impl ScreenSprinkler {
    pub fn new() -> Self {
        Self {
            schedule: Select::<{ScheduleController::SIZE}>::new(),
            zone: Select::<{ZoneController::SIZE}>::new()
        }
    }
    
    #[inline]
    fn apply_pending_draw(display_signal: &mut EventBits) {
        if UPDATE_DRAW.load(Ordering::SeqCst) {
            UPDATE_DRAW.store(false, Ordering::SeqCst);
            *display_signal |= DisplayFlag::Draw as u32;
        }
    }


    fn draw_schedule_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> osal_rs::utils::Result<()> {

        

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
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str("WiFi Auth"),
            param,
            Some(|_, confirmed| {
                // if confirmed {
                //     Self::set_state_with_old(FSMState::Schedule);
                // } else {
                //     Self::set_state_with_old(FSMState::Schedule);
                // }
            }),
        )?;



        Ok(())
    }

    fn draw_zone_state(
        &mut self,
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits,
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
    ) -> osal_rs::utils::Result<()> {

        Self::apply_pending_draw(display_signal);
        Ok(())  
    }

}