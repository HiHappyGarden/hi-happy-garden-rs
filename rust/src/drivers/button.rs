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

use alloc::sync::Arc;
use alloc::boxed::Box;
use once_cell::race::OnceBox;

use osal_rs::os::types::{StackType, TickType};
use osal_rs::{log_error, log_info};
use osal_rs::os::{EventGroup, EventGroupFn, Mutex, MutexFn, System, SystemFn, Thread, ThreadFn, ThreadParam, Timer, TimerFn};
use osal_rs::utils::{Error, OsalRsBool, Result};

use crate::drivers::gpio::{InterruptCallback, InterruptType};
use crate::drivers::platform::{self, GPIO_CONFIG_SIZE, Gpio, GpioPeripheral, OsalThreadPriority};
use crate::traits::button::{ButtonCallback, ButtonState, OnClickable};
use crate::traits::state::Initializable;

use button_events::*;

const APP_TAG: &str = "Button";
const APP_THREAD_NAME: &str = "button_trd";
const APP_STACK_SIZE: StackType = 512;
const APP_DEBOUNCE_TIME: TickType = 50;


static EVENT_HANDLER: OnceBox<Arc<EventGroup>> = OnceBox::new();
static BUTTON_STATE: AtomicU32 = AtomicU32::new(0);

pub mod button_events {
    use osal_rs::os::types::EventBits;

    pub const BUTTON_NONE: EventBits = 0x0000;
    pub const BUTTON_PRESSED: EventBits = 0x0001;
    pub const BUTTON_RELEASED: EventBits = 0x0002;
}

pub struct Button {
    gpio_ref: GpioPeripheral,
    gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>,
    thread: Thread,
    param: Option<ThreadParam>,
    callback: Arc<Mutex<Option<Box<ButtonCallback>>>>,
}




extern "C" fn button_isr() {
    let event_handler = EVENT_HANDLER.get_or_init(|| Box::new(Arc::new(EventGroup::new().unwrap())));

    let state = BUTTON_STATE.load(Ordering::Relaxed);

    if state == BUTTON_NONE || state & BUTTON_RELEASED == BUTTON_RELEASED {
        BUTTON_STATE.store(BUTTON_PRESSED, Ordering::Relaxed);
        event_handler.set_from_isr(BUTTON_PRESSED).unwrap();
    } else if state & BUTTON_PRESSED == BUTTON_PRESSED {
        BUTTON_STATE.store(BUTTON_RELEASED, Ordering::Relaxed);
        event_handler.set_from_isr(BUTTON_RELEASED).unwrap();
    }
}

impl OnClickable for Button {
    fn set_on_click(&mut self, callback: Box<ButtonCallback>) {
        if let Ok(mut cb) = self.callback.lock() {
            *cb = Some(callback);
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
    pub fn new(gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Self {

        let event_handler = EVENT_HANDLER.get_or_init(|| Box::new(Arc::new(EventGroup::new().unwrap())));

        Self {
            gpio_ref: GpioPeripheral::Btn,
            gpio,
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, OsalThreadPriority::Normal),
            param: Some(Arc::clone(event_handler) as ThreadParam),
            callback: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn init(&mut self, gpio: &mut Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Result<()> {
        log_info!(APP_TAG, "Init button");

        if gpio.lock()?.set_interrupt(&self.gpio_ref, InterruptType::BothEdge, true, button_isr) == OsalRsBool::False {
            log_error!(APP_TAG, "Error setting button interrupt");
            return Err(Error::NotFound);
        }

        

        let event_handler = if let Some(param) = &self.param {
            param.clone().downcast::<EventGroup>().map_err(|_| {
                log_error!(APP_TAG, "Error downcasting event handler");
                Error::InvalidType
            })?
        } else {
            log_error!(APP_TAG, "No event handler provided");
            return Err(Error::NullPtr);
        };

        let callback = Arc::clone(&self.callback);
        self.thread.spawn(Some(event_handler as ThreadParam), move |_thread, _param| {
            let event_handler = EVENT_HANDLER.get_or_init(|| Box::new(Arc::new(EventGroup::new().unwrap())));

            

            let mut debounce: TickType = 0;
            loop {
                
                let bits = event_handler.wait(BUTTON_PRESSED | BUTTON_RELEASED, TickType::MAX) & 0x0003;
                event_handler.clear(bits);

                if debounce != 0 && System::get_tick_count() - debounce < APP_DEBOUNCE_TIME {
                    continue;
                }
                if bits & BUTTON_PRESSED == BUTTON_PRESSED {
                    if let Ok(cb) = callback.lock() {
                        if let Some(ref c) = *cb {
                            c(ButtonState::Pressed);
                        }
                    }
                } else if bits & BUTTON_RELEASED == BUTTON_RELEASED {
                    if let Ok(cb) = callback.lock() {
                        if let Some(ref c) = *cb {
                            c(ButtonState::Released);
                        }
                    }
                }

                debounce = System::get_tick_count();
            }

        })?;

        Ok(())
    }
}

