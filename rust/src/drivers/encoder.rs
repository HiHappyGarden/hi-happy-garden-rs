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
use osal_rs::{arcmux, log_error, log_info, log_warning, minimal_stack_size};
use osal_rs::os::types::{StackType, TickType};
use osal_rs::os::{EventGroup, EventGroupFn, Mutex, MutexFn, System, SystemFn, Thread, ThreadFn, ThreadParam};
use osal_rs::utils::{ArcMux, Error, OsalRsBool, Result};

use crate::drivers::gpio::{InterruptType};
use crate::drivers::platform::{self, GPIO_CONFIG_SIZE, Gpio, GpioPeripheral, OsalThreadPriority};
use crate::traits::button::{ButtonState, OnClickable};
use crate::traits::encoder::{EncoderDirection, OnRotatableAndClickable, SetRotatableAndClickable};
use crate::traits::state::Initializable;
use encoder_events::*;

const APP_TAG: &str = "Encoder";
const APP_THREAD_NAME: &str = "encoder_trd";
const APP_STACK_SIZE: StackType = 1_024;
const APP_DEBOUNCE_TIME: TickType = 3;

static ENCODER_EVENTS: OnceBox<Arc<EventGroup>> = OnceBox::new();
static ENCODER_STATE: AtomicU32 = AtomicU32::new(0);
static ENCODER_POSITION: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);

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
    gpio: ArcMux<Gpio<GPIO_CONFIG_SIZE>>,
    thread: Thread,
    rotable_and_clickable: ArcMux<Option<ArcMux<dyn OnRotatableAndClickable>>>
}

extern "C" fn encoder_button_isr() {
    let encoder_events = ENCODER_EVENTS.get().unwrap();

    let state = ENCODER_STATE.load(Ordering::Relaxed);
    
    // Clear button bits and preserve encoder rotation bits
    let rotation_bits = state & (ENCODER_CCW_RISE | ENCODER_CCW_FALL | ENCODER_CW_RISE | ENCODER_CW_FALL);

    if state & ENCODER_PRESSED != ENCODER_PRESSED {
        // Button not pressed, so this is a press event
        ENCODER_STATE.store(rotation_bits | ENCODER_PRESSED, Ordering::Relaxed);
        let _ = encoder_events.set_from_isr(ENCODER_PRESSED);
    } else {
        // Button was pressed, so this is a release event
        ENCODER_STATE.store(rotation_bits | ENCODER_RELEASED, Ordering::Relaxed);
        let _ = encoder_events.set_from_isr(ENCODER_RELEASED);
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
        let _ = encoder_events.set_from_isr(ENCODER_CCW_FALL);
    } else {
        // Last state was FALL, so this is a RISE event
        ENCODER_STATE.store(other_bits | ENCODER_CCW_RISE, Ordering::Relaxed);
        let _ = encoder_events.set_from_isr(ENCODER_CCW_RISE);
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
        let _ = encoder_events.set_from_isr(ENCODER_CW_FALL);
    } else {
        // Last state was FALL, so this is a RISE event
        ENCODER_STATE.store(other_bits | ENCODER_CW_RISE, Ordering::Relaxed);
        let _ = encoder_events.set_from_isr(ENCODER_CW_RISE);
    }
}


impl Encoder {
    pub fn new(gpio: ArcMux<Gpio<GPIO_CONFIG_SIZE>>) -> Self {
        let _ = ENCODER_EVENTS.get_or_init(|| Box::new(Arc::new(EventGroup::new().unwrap())));
                
        Self {
            gpio_ccw_ref: GpioPeripheral::EncoderCCW,
            gpio_cw_ref: GpioPeripheral::EncoderCW,
            gpio_btn_ref: GpioPeripheral::EncoderBtn,
            gpio,
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_STACK_SIZE, OsalThreadPriority::Normal),
            rotable_and_clickable: arcmux!(None)
        }
    }

    pub fn init(&mut self, gpio: &mut ArcMux<Gpio<GPIO_CONFIG_SIZE>>) -> Result<()> {
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


        let rotable_and_clickable = ArcMux::clone(&self.rotable_and_clickable);
        let gpio_clone = ArcMux::clone(gpio);
        let gpio_ccw_ref = self.gpio_ccw_ref;
        let gpio_cw_ref = self.gpio_cw_ref;
        
        self.thread.spawn_simple( move || {

            let event_handler = ENCODER_EVENTS.get().unwrap();

            let mut debounce: TickType = 0;
            // State tracking: use 2-bit encoding where bit1=CCW, bit0=CW
            let mut last_state: u8 = 0;
            
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
                
                // Handle button press/release
                let mut encoder_pressed = ButtonState::None; 
                if bits & ENCODER_PRESSED == ENCODER_PRESSED {
                    encoder_pressed = ButtonState::Pressed;
                } else if bits & ENCODER_RELEASED == ENCODER_RELEASED {
                    encoder_pressed = ButtonState::Released;
                }
                
                if encoder_pressed != ButtonState::None {
                    if let Ok(rotable_and_clickable) = rotable_and_clickable.lock() {
                        if let Some(ref rotable_and_clickable) = *rotable_and_clickable {
                            if let Ok(mut rotable_and_clickable) = rotable_and_clickable.lock() {
                                rotable_and_clickable.on_click(encoder_pressed);
                            } else {
                                log_error!(APP_TAG, "Reference empty");
                            }
                        } else {
                            log_warning!(APP_TAG, "No callback set for encoder button pressed");
                        }
                    }
                }


                // Handle encoder rotation - read actual GPIO state
                // Only process rotation events
                if bits & (ENCODER_CCW_RISE | ENCODER_CCW_FALL | ENCODER_CW_RISE | ENCODER_CW_FALL) != 0 {
                    // Read current state from GPIO pins
                    let current_state = if let Ok(gpio_lock) = gpio_clone.lock() {
                        let ccw_pin = gpio_lock.read(&gpio_ccw_ref).unwrap_or(0);
                        let cw_pin = gpio_lock.read(&gpio_cw_ref).unwrap_or(0);
                        ((ccw_pin & 1) << 1) | (cw_pin & 1)
                    } else {
                        continue;
                    };
                    
                    // Decode direction using state transition table
                    // Clockwise sequence:  00 -> 01 -> 11 -> 10 -> 00
                    // Counter-clockwise:   00 -> 10 -> 11 -> 01 -> 00
                    let direction: Option<EncoderDirection> = match (last_state, current_state as u8) {
                        // Clockwise transitions
                        (0b00, 0b01) | (0b01, 0b11) | (0b11, 0b10) | (0b10, 0b00) => {
                            Some(EncoderDirection::Clockwise)
                        }
                        // Counter-clockwise transitions
                        (0b00, 0b10) | (0b10, 0b11) | (0b11, 0b01) | (0b01, 0b00) => {
                            Some(EncoderDirection::CounterClockwise)
                        }
                        // Invalid or no change
                        _ => None
                    };
                    
                    last_state = current_state as u8;
                    
                    // If we detected a rotation, update position and call callback
                    if let Some(dir) = direction {
                        let position = match dir {
                            EncoderDirection::Clockwise => {
                                ENCODER_POSITION.fetch_add(1, Ordering::Relaxed) + 1
                            }
                            EncoderDirection::CounterClockwise => {
                                ENCODER_POSITION.fetch_sub(1, Ordering::Relaxed) - 1
                            }
                        };
                        
                        if let Ok(rotable_and_clickable) = rotable_and_clickable.lock() {
                            if let Some(ref rotable_and_clickable) = *rotable_and_clickable {
                                if let Ok(mut rotable_and_clickable) = rotable_and_clickable.lock() {
                                    rotable_and_clickable.on_rotable(dir, position);
                                } else {
                                    log_error!(APP_TAG, "Reference empty");
                                }
                            } else {
                                log_warning!(APP_TAG, "No callback set for encoder rotation encoder {:?} position: {}", dir, position);
                            }
                        }
                    }
                }

                debounce = System::get_tick_count();
            }

        })?;



        Ok(())
    }
}


impl SetRotatableAndClickable for Encoder {
    fn set_on_rotate_and_click(&mut self, value: ArcMux<dyn OnRotatableAndClickable>) {
        if let Ok(mut rotable_and_clickable) = self.rotable_and_clickable.lock() {
            *rotable_and_clickable = Some(value);
        }
    }   

    fn get_state(&self) -> ButtonState {
        let state = ENCODER_STATE.load(Ordering::Relaxed);
        match state {
            x if x & ENCODER_PRESSED == ENCODER_PRESSED => ButtonState::Pressed,
            x if x & ENCODER_RELEASED == ENCODER_RELEASED => ButtonState::Released,
            _ => ButtonState::None,
        }
    }

    fn get_position(&self) -> i32 {
        ENCODER_POSITION.load(Ordering::Relaxed)
    }
}