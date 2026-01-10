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

#![allow(dead_code)]

use core::cell::RefCell;
use core::ptr::fn_addr_eq;
use core::str;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::time::Duration;

use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::boxed::Box;
use once_cell::race::OnceBox;

use osal_rs::os::types::{StackType, TickType};
use osal_rs::{arcmux, log_error, log_info, log_warning};
use osal_rs::os::{EventGroup, EventGroupFn, Mutex, MutexFn, System, SystemFn, Thread, ThreadFn, ThreadParam, Timer, TimerFn, RawMutexFn};
use osal_rs::utils::{ArcMux, Error, OsalRsBool, Result};
use osal_rs_tests::freertos::event_group_tests;

use crate::drivers::gpio::{InterruptCallback, InterruptType};
use crate::drivers::platform::{self, GPIO_CONFIG_SIZE, Gpio, GpioPeripheral, ThreadPriority};
use crate::traits::button::{ButtonState, OnClickable, SetClickable};
use crate::traits::state::Initializable;

use button_events::*;

const APP_TAG: &str = "Button";
const APP_THREAD_NAME: &str = "button_trd";
const APP_STACK_SIZE: StackType = 512;
const APP_DEBOUNCE_TIME: TickType = 50;


static BUTTON_EVENTS: OnceBox<EventGroup> = OnceBox::new();
static BUTTON_STATE: AtomicU32 = AtomicU32::new(0);

pub mod button_events {
    use osal_rs::os::types::EventBits;

    pub const BUTTON_NONE: EventBits = 0x00_00;
    pub const BUTTON_PRESSED: EventBits = 0x00_01;
    pub const BUTTON_RELEASED: EventBits = 0x00_02;
}

pub struct Button {
    gpio_ref: GpioPeripheral,
    thread: Thread,
    clickable: Option<ArcMux<&'static dyn OnClickable>>
}




extern "C" fn button_isr() {
    let event_handler = BUTTON_EVENTS.get().unwrap();

    let state = BUTTON_STATE.load(Ordering::Relaxed);

    if state == BUTTON_NONE || state & BUTTON_RELEASED == BUTTON_RELEASED {
        BUTTON_STATE.store(BUTTON_PRESSED, Ordering::Relaxed);
        event_handler.set_from_isr(BUTTON_PRESSED).unwrap();
    } else if state & BUTTON_PRESSED == BUTTON_PRESSED {
        BUTTON_STATE.store(BUTTON_RELEASED, Ordering::Relaxed);
        event_handler.set_from_isr(BUTTON_RELEASED).unwrap();
    }
}

impl SetClickable<'static> for Button {
    fn set_on_click(&mut self, value: &'static dyn OnClickable) {
        self.clickable = Some(arcmux!(value));
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


impl Button {
    pub fn new() -> Self {
        
        Self {
            gpio_ref: GpioPeripheral::Btn,
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, ThreadPriority::Normal),
            clickable: None,
        }
    }
    
    pub fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init button");

        let mut gpio = Gpio::new();

        gpio.get_mutex().lock();
        if gpio.set_interrupt(&self.gpio_ref, InterruptType::BothEdge, true, button_isr) == OsalRsBool::False {
            log_error!(APP_TAG, "Error setting button interrupt");
            return Err(Error::NotFound);
        }
        gpio.get_mutex().unlock();


        if let Ok(event_group) = EventGroup::new() {
            let _ = BUTTON_EVENTS.set(Box::new(event_group));
        } else {
            log_error!(APP_TAG, "Error creating button event group");
            return Err(Error::OutOfMemory);
        }

                
        let clickable = self.clickable.clone();
        self.thread.spawn_simple( move || {
            let event_handler = BUTTON_EVENTS.get().unwrap();



            let mut debounce: TickType = 0;
            loop {
                
                let bits = event_handler.wait(BUTTON_PRESSED | BUTTON_RELEASED, TickType::MAX);
                event_handler.clear(bits);

                let clickable = if let Some(clickable) = clickable.as_ref() {
                    match  clickable.lock() {
                        Ok(c) => c,
                        Err(_) => continue,
                    }
                } else {
                    continue;
                };

                if debounce != 0 && System::get_tick_count() - debounce < APP_DEBOUNCE_TIME {
                    continue;
                }
                
                let state = if bits & BUTTON_PRESSED == BUTTON_PRESSED {
                    ButtonState::Pressed
                } else if bits & BUTTON_RELEASED == BUTTON_RELEASED {
                    ButtonState::Released
                } else {
                    continue;
                };

                
                clickable.on_click(state);
      

                debounce = System::get_tick_count();
            }
                    

        })?;

        Ok(())
    }
}

