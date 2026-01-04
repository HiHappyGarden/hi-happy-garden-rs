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
 
 use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};

use alloc::boxed::Box;
use alloc::sync::Arc;

use once_cell::race::OnceBox;
use osal_rs::{log_error, log_info, minimal_stack_size};
use osal_rs::os::types::{StackType, TickType};
use osal_rs::os::{EventGroup, EventGroupFn, Mutex, MutexFn, System, SystemFn, Thread, ThreadFn, ThreadParam};
use osal_rs::utils::{Error, OsalRsBool, Result};

use crate::drivers::gpio::{InterruptType};
use crate::drivers::platform::{self, GPIO_CONFIG_SIZE, Gpio, GpioPeripheral, OsalThreadPriority};
use crate::traits::button::{ButtonCallback, ButtonState};
use crate::traits::state::Initializable;
use encoder_events::*;

const APP_TAG: &str = "Encoder";
const APP_THREAD_NAME: &str = "encoder_trd";
const APP_STACK_SIZE: StackType = 1_024;
const APP_DEBOUNCE_TIME: TickType = 3;

static ENCODER_EVENTS: OnceBox<Arc<EventGroup>> = OnceBox::new();
static ENCODER_STATE: AtomicU32 = AtomicU32::new(0);

pub mod encoder_events {
    use osal_rs::os::types::EventBits;

    #[allow(dead_code)]
    pub const ENCODER_NONE: EventBits = 0x00_00;
    pub const ENCODER_PRESSED: EventBits = 0x00_01;
    pub const ENCODER_RELEASED: EventBits = 0x00_02;
    pub const ENCODER_CCW_RISE: EventBits = 0x00_04;
    pub const ENCODER_CCW_FALL: EventBits = 0x00_08;
    pub const ENCODER_CW_RISE: EventBits = 0x00_10;
    pub const ENCODER_CW_FALL: EventBits = 0x00_20;
}


#[allow(dead_code)]
pub struct Encoder {
    gpio_ccw_ref: GpioPeripheral,
    gpio_cw_ref: GpioPeripheral,
    gpio_btn_ref: GpioPeripheral,
    gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>,
    thread: Thread,
    callback: Arc<Mutex<Option<Box<ButtonCallback>>>>,
    
}

extern "C" fn encoder_button_isr() {
    let encoder_events = ENCODER_EVENTS.get().unwrap();

    let state = ENCODER_STATE.load(Ordering::Relaxed);
    
    // Clear button bits and preserve encoder rotation bits
    let rotation_bits = state & (ENCODER_CCW_RISE | ENCODER_CCW_FALL | ENCODER_CW_RISE | ENCODER_CW_FALL);

    if state & ENCODER_PRESSED != ENCODER_PRESSED {
        // Button not pressed, so this is a press event
        ENCODER_STATE.store(rotation_bits | ENCODER_PRESSED, Ordering::Relaxed);
        encoder_events.set_from_isr(ENCODER_PRESSED).unwrap();
    } else {
        // Button was pressed, so this is a release event
        ENCODER_STATE.store(rotation_bits | ENCODER_RELEASED, Ordering::Relaxed);
        encoder_events.set_from_isr(ENCODER_RELEASED).unwrap();
    }
}

extern "C" fn encoder_ccw_isr() {
    let encoder_events = ENCODER_EVENTS.get().unwrap();

    let state = ENCODER_STATE.load(Ordering::Relaxed);
    
    // Clear CCW bits and preserve button and CW bits
    let other_bits = state & (ENCODER_PRESSED | ENCODER_RELEASED | ENCODER_CW_RISE | ENCODER_CW_FALL);

    if state & ENCODER_CCW_FALL != ENCODER_CCW_FALL {
        // Last state was not FALL (or NONE/RISE), so this is a FALL event
        ENCODER_STATE.store(other_bits | ENCODER_CCW_FALL, Ordering::Relaxed);
        encoder_events.set_from_isr(ENCODER_CCW_FALL).unwrap();
    } else {
        // Last state was FALL, so this is a RISE event
        ENCODER_STATE.store(other_bits | ENCODER_CCW_RISE, Ordering::Relaxed);
        encoder_events.set_from_isr(ENCODER_CCW_RISE).unwrap();
    }
}

extern "C" fn encoder_cw_isr() {
    let encoder_events = ENCODER_EVENTS.get().unwrap();

    let state = ENCODER_STATE.load(Ordering::Relaxed);
    
    // Clear CW bits and preserve button and CCW bits
    let other_bits = state & (ENCODER_PRESSED | ENCODER_RELEASED | ENCODER_CCW_RISE | ENCODER_CCW_FALL);

    if state & ENCODER_CW_FALL != ENCODER_CW_FALL {
        // Last state was not FALL (or NONE/RISE), so this is a FALL event
        ENCODER_STATE.store(other_bits | ENCODER_CW_FALL, Ordering::Relaxed);
        encoder_events.set_from_isr(ENCODER_CW_FALL).unwrap();
    } else {
        // Last state was FALL, so this is a RISE event
        ENCODER_STATE.store(other_bits | ENCODER_CW_RISE, Ordering::Relaxed);
        encoder_events.set_from_isr(ENCODER_CW_RISE).unwrap();
    }
}


impl Encoder {
    pub fn new(gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Self {
        let _ = ENCODER_EVENTS.get_or_init(|| Box::new(Arc::new(EventGroup::new().unwrap())));
                
        Self {
            gpio_ccw_ref: GpioPeripheral::EncoderCCw,
            gpio_cw_ref: GpioPeripheral::EncoderCW,
            gpio_btn_ref: GpioPeripheral::EncoderBtn,
            gpio,
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, OsalThreadPriority::Normal),
            callback: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init(&mut self, gpio: &mut Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Result<()> {
        log_info!(APP_TAG, "Init encoder");


        if gpio.lock()?.set_interrupt(&self.gpio_ccw_ref, InterruptType::BothEdge, true, encoder_ccw_isr) == OsalRsBool::False {
            log_error!(APP_TAG, "Error setting CCW interrupt");
            return Err(Error::NotFound);
        }

        if gpio.lock()?.set_interrupt(&self.gpio_cw_ref, InterruptType::BothEdge, true, encoder_cw_isr) == OsalRsBool::False {
            log_error!(APP_TAG, "Error setting CW interrupt");
            return Err(Error::NotFound);
        }

        if gpio.lock()?.set_interrupt(&self.gpio_btn_ref, InterruptType::BothEdge, true, encoder_button_isr) == OsalRsBool::False {
            log_error!(APP_TAG, "Error setting Button interrupt");
            return Err(Error::NotFound);
        }


        let callback = Arc::clone(&self.callback);
        self.thread.spawn_simple( move || {

            let event_handler = ENCODER_EVENTS.get().unwrap();

            let mut debounce: TickType = 0;
            loop {
                
                let bits = event_handler.wait(
                    ENCODER_PRESSED | ENCODER_RELEASED | 
                    ENCODER_CCW_RISE | ENCODER_CCW_FALL | 
                    ENCODER_CW_RISE | ENCODER_CW_FALL, 
                    TickType::MAX
                );
                event_handler.clear(bits);

                if debounce != 0 && System::get_tick_count() - debounce < APP_DEBOUNCE_TIME {
                    continue;
                }
                if bits & ENCODER_PRESSED == ENCODER_PRESSED {
                    if let Ok(cb) = callback.lock() {
                        if let Some(ref c) = *cb {
                            c(ButtonState::Pressed);
                        } else {
                            log_error!(APP_TAG, "No callback set for encoder button press");
                        }
                    }
                } else if bits & ENCODER_RELEASED == ENCODER_RELEASED {
                    if let Ok(cb) = callback.lock() {
                        if let Some(ref c) = *cb {
                            c(ButtonState::Released);
                        } else {
                            log_error!(APP_TAG, "No callback set for encoder button release");
                        }
                    }
                } else if bits & ENCODER_CCW_RISE == ENCODER_CCW_RISE {
                    log_info!(APP_TAG, "Encoder CCW Rise detected");
                } else if bits & ENCODER_CCW_FALL == ENCODER_CCW_FALL {
                    log_info!(APP_TAG, "Encoder CCW Fall detected");
                } else if bits & ENCODER_CW_RISE == ENCODER_CW_RISE {
                    log_info!(APP_TAG, "Encoder CW Rise detected");
                } else if bits & ENCODER_CW_FALL == ENCODER_CW_FALL {
                    log_info!(APP_TAG, "Encoder CW Fall detected");
                }

                debounce = System::get_tick_count();
            }

        })?;



        Ok(())
    }
}
