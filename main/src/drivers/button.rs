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

use core::str;
use core::sync::atomic::{AtomicU32, Ordering};

use osal_rs::os::types::{StackType, TickType};
use osal_rs::os::{EventGroup, EventGroupFn, RawMutexFn, System, SystemFn, Thread, ThreadFn};
use osal_rs::utils::{Error, OsalRsBool, Result};
use osal_rs::{log_error, log_info};

use crate::drivers::gpio::{Gpio, InterruptType};
use crate::drivers::platform::{GpioPeripheral, ThreadPriority};

use crate::traits::button::{ButtonState, OnClickable, SetClickable};

use button_events::*;

const APP_TAG: &str = "Button";
const APP_THREAD_NAME: &str = "button_trd";
const APP_STACK_SIZE: StackType = 512;
const APP_DEBOUNCE_TIME: TickType = 50;


pub mod button_events {
    use osal_rs::os::types::EventBits;

    pub const BUTTON_NONE: EventBits = 0x00_00;
    pub const BUTTON_PRESSED: EventBits = 0x00_01;
    pub const BUTTON_RELEASED: EventBits = 0x00_02;
}

static BUTTON_STATE: AtomicU32 = AtomicU32::new(0);
static mut EVENT_HANDLER: Option<EventGroup> = None;

const fn event_handler() -> &'static EventGroup {
    unsafe {
        match &*&raw const EVENT_HANDLER {
            Some(event_handler) => event_handler,
            None => panic!("EVENT_HANDLER is not initialized"),    
        }
    }
}

pub struct Button {
    gpio_ref: GpioPeripheral,
    thread: Thread,
}




extern "C" fn button_isr() {
    let event_handler = event_handler();

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
    fn set_on_click(&mut self, clickable: &'static dyn OnClickable) {

        let ret = self.thread.spawn_simple( move || {
            let event_handler = event_handler();

            let mut debounce: TickType = 0;
            loop {
                
                let bits = event_handler.wait(BUTTON_PRESSED | BUTTON_RELEASED, TickType::MAX);
                event_handler.clear(bits);


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
                    

        });

        if let Err(e) = ret {
            log_error!(APP_TAG, "Error spawning button thread: {:?}", e);
        }

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
            unsafe {
                EVENT_HANDLER = Some(event_group);
            }
        } else {
            log_error!(APP_TAG, "Error creating button event group");
            return Err(Error::OutOfMemory)
        }

        Ok(())
    }
}

