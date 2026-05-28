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
use core::sync::atomic::{AtomicI8, Ordering};

use alloc::sync::Arc;
use osal_rs::os::Mutex;
use osal_rs::os::types::EventBits;
use osal_rs::utils::{Bytes, Error, Result};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::display::text::Text;
use crate::apps::signals::display::DisplayFlag;
use crate::traits::lcd_display::LCDDisplayFn;
use crate::traits::rtc::RTC;
use crate::traits::screen::{Screen, ScreenParam, ScreenRoute};

static SELECTED_SCREEN: AtomicI8 = AtomicI8::new(-1);
static PENDING_SELECTION: AtomicI8 = AtomicI8::new(-1);

fn select_callback(_: Option<ScreenParam<u16>>, confirmed: bool) {
    if confirmed {
        SELECTED_SCREEN.store(PENDING_SELECTION.load(Ordering::SeqCst), Ordering::SeqCst);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FSMState {
    Info,
    DateTime,
    DaylightSavingTime,
    Wifi,
    User,
}

impl From<i8> for FSMState {
    fn from(value: i8) -> Self {
        match value {
            0 => FSMState::Info,
            1 => FSMState::DateTime,
            2 => FSMState::DaylightSavingTime,
            3 => FSMState::Wifi,
            4 => FSMState::User,
            _ => FSMState::Info, // Default case
        }
    }
}

impl From<FSMState> for i8 {
    fn from(state: FSMState) -> Self {
        match state {
            FSMState::Info => 0,
            FSMState::DateTime => 1,
            FSMState::DaylightSavingTime => 2,
            FSMState::Wifi => 3,
            FSMState::User => 4,
        }
    }
}

impl FSMState {
    fn label(self) -> &'static str {
        match self {
            FSMState::Info             => "Info",
            FSMState::DateTime         => "Date Time",
            FSMState::DaylightSavingTime => "Daylight Saving Time",
            FSMState::Wifi             => "Wifi",
            FSMState::User             => "User",
        }
    }
}

 pub(super) struct ScreenMain {
    fsm_state: FSMState,
    text: Text,
    value: i8,
}

impl ScreenRoute for ScreenMain {
    #[allow(unused)]
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[allow(unused)]
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn draw(&mut self, 
        lcd: &mut dyn LCDDisplayFn,
        display_signal: &mut EventBits, 
        _status_signal: &mut EventBits, 
        rtc: &Arc<Mutex<dyn RTC + 'static>>,
        
    ) -> Result<()> {

        self.update_input(display_signal);

        PENDING_SELECTION.store(self.fsm_state.into(), Ordering::SeqCst);
        self.text.draw(
            lcd,
            display_signal,
            rtc,
            &Bytes::<DISPLAY_INPUT_MAX_SIZE>::from_str(self.fsm_state.label()),
            ScreenParam::<u16>::default(),
            Some(select_callback),
        )?;

        

        if SELECTED_SCREEN.load(Ordering::SeqCst) != -1 {
            self.value = SELECTED_SCREEN.load(Ordering::SeqCst);
            SELECTED_SCREEN.store(-1, Ordering::SeqCst);
            Ok(())
        } else {
            Err(Error::ReturnWithCode(1))
        }
        
    }
}

impl ScreenMain {
    pub(super) fn new(fsm_state: FSMState) -> Self {
        Self {
            fsm_state,
            text: Text::new(),
            value: -1,
        }
    }

    fn update_input(&mut self, signal: &mut EventBits) {

        if *signal & DisplayFlag::EncoderRotatedClockwise as u32 != 0 {
            self.fsm_state = match self.fsm_state {
                FSMState::Info => FSMState::DateTime,
                FSMState::DateTime => FSMState::DaylightSavingTime,
                FSMState::DaylightSavingTime => FSMState::Wifi,
                FSMState::Wifi => FSMState::User,
                FSMState::User => FSMState::Info,
            };
            *signal |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn
        } else  if *signal & DisplayFlag::EncoderRotatedCounterClockwise as u32 != 0 {
            self.fsm_state = match self.fsm_state {
                FSMState::Info => FSMState::User,
                FSMState::DateTime => FSMState::Info,
                FSMState::DaylightSavingTime => FSMState::DateTime,
                FSMState::Wifi => FSMState::DaylightSavingTime,
                FSMState::User => FSMState::Wifi,
            };
            *signal |= DisplayFlag::Draw as u32; // Set the flag to indicate that the display should be redrawn
        }
    }

    #[allow(dead_code)]
    pub(super) fn get_selected_screen(&self) -> Option<FSMState> {
        if self.value != -1 {
            Some(FSMState::from(self.value))
        } else {
            None
        }
    }
}