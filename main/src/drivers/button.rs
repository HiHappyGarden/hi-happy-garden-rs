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


use core::str;
use core::sync::atomic::{AtomicBool, Ordering};

use osal_rs::os::types::{StackType, TickType};
use osal_rs::os::{EventGroup, EventGroupFn, System, SystemFn, Thread, ThreadFn};
use osal_rs::utils::{Error, OnceBox, OsalRsBool, Result};
use osal_rs::{log_error, log_info, log_warning};

use crate::drivers::gpio::{Gpio, InterruptType};
use crate::drivers::platform::{GpioPeripheral, ThreadPriority};

use crate::traits::button::{ButtonState, OnClickable, SetClickable};
use crate::traits::signal::{AtomicEventBits, AtomicTickType};
use crate::traits::state::Initializable;

use button_events::*;

const APP_TAG: &str = "Button";
const THREAD_NAME: &str = "button_thr";
const STACK_SIZE: StackType = 256;
const DEBOUNCE_TIME: TickType = 50;


pub mod button_events {
    use osal_rs::os::types::EventBits;

    pub const BUTTON_NONE: EventBits = 0x00_00;
    pub const BUTTON_PRESSED: EventBits = 0x00_01;
    pub const BUTTON_RELEASED: EventBits = 0x00_02;
}

static BUTTON_STATE: AtomicEventBits = AtomicEventBits::new(0);
static LAST_INTERRUPT_TIME: AtomicTickType = AtomicTickType::new(0);
static EVENT_HANDLER: OnceBox<EventGroup> = OnceBox::new();

pub struct Button {
    gpio_ref: GpioPeripheral,
    thread: Thread,
    thread_started: AtomicBool,
}




extern "C" fn button_isr() {
    let Some(button_events) = EVENT_HANDLER.get() else { return };

    let current_time = System::get_tick_count();
    let last_time = LAST_INTERRUPT_TIME.load(Ordering::Relaxed);
    
    // Debounce
    if current_time.saturating_sub(last_time) < DEBOUNCE_TIME {
        return;
    }
    
    LAST_INTERRUPT_TIME.store(current_time, Ordering::Relaxed);

    let state = BUTTON_STATE.load(Ordering::Relaxed);

    if state == BUTTON_NONE || state & BUTTON_RELEASED == BUTTON_RELEASED {
        BUTTON_STATE.store(BUTTON_PRESSED, Ordering::Relaxed);
        button_events.set_from_isr(BUTTON_PRESSED).unwrap();
    } else if state & BUTTON_PRESSED == BUTTON_PRESSED {
        BUTTON_STATE.store(BUTTON_RELEASED, Ordering::Relaxed);
        button_events.set_from_isr(BUTTON_RELEASED).unwrap();
    }
}

impl SetClickable<'static> for Button {
    fn set_on_click(&mut self, clickable: &'static dyn OnClickable) {
        // Check if thread is already running
        if self.thread_started.load(Ordering::Acquire) {
            log_warning!(APP_TAG, "Button thread already started, ignoring new callback");
            return;
        }

        // Mark thread as started
        self.thread_started.store(true, Ordering::Release);

        let ret = self.thread.spawn_simple( move || {
            let Some(event_handler) = EVENT_HANDLER.get() else { return Err(Error::OutOfMemory) };

            loop {
                
                let bits = event_handler.wait(BUTTON_PRESSED | BUTTON_RELEASED, false, TickType::MAX);
                event_handler.clear(bits);
                
                let state = if bits & BUTTON_PRESSED == BUTTON_PRESSED {
                    ButtonState::Pressed
                } else if bits & BUTTON_RELEASED == BUTTON_RELEASED {
                    ButtonState::Released
                } else {
                    continue;
                };

                clickable.on_click(state);
            }
        });

        if let Err(e) = &ret {
            log_error!(APP_TAG, "Error spawning button thread: {:?}", e);
            self.thread_started.store(false, Ordering::Release);
        }

        self.thread = ret.unwrap();
    }

    fn get_state(&self) -> ButtonState {
        let state = BUTTON_STATE.load(Ordering::Relaxed);
        match state {
            x if x & BUTTON_PRESSED == BUTTON_PRESSED => ButtonState::Pressed,
            x if x & BUTTON_RELEASED == BUTTON_RELEASED => ButtonState::Released,
            _ => ButtonState::None,
        }
    }
}

impl Initializable for Button {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init button");

        EVENT_HANDLER.get_or_try_init(EventGroup::new)?;

        let mut gpio = Gpio::shared();

        if gpio.set_interrupt(&self.gpio_ref, InterruptType::BothEdge, true, button_isr) == OsalRsBool::False {
            log_error!(APP_TAG, "Error setting button interrupt");
            return Err(Error::NotFound);
        }




        Ok(())
    }
}

impl Button {
    pub fn shared() -> Self {
        
        Self {
            gpio_ref: GpioPeripheral::Btn,
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal),
            thread_started: AtomicBool::new(false),
        }
    }

}

