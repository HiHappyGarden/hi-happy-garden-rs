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

use core::sync::atomic::{AtomicU16, Ordering};
use core::time::Duration;

use osal_rs::log_info;
use osal_rs::os::{System, Thread, ThreadFn};
use osal_rs::os::types::StackType;

use crate::drivers::platform::ThreadPriority;
use crate::drivers::rgb_led::RgbLed;
use crate::traits::rgb_led::{Color, RgbLed as _};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;

use crate::apps::signals::status::{StatusFlag::{self, *}, StatusSignal};

const APP_TAG: &str = "AppSystemLed";
const THREAD_NAME: &str = "system_led_trd";
const STACK_SIZE: StackType = 256;
const BLINK_INTERVAL_MS: u16 = 500;
const TICK_INTERVAL_MS: u16 = 100; 
static TIMER: AtomicU16 = AtomicU16::new(0);

const COLOR_RED: Color = Color::new(255, 0, 0);
const COLOR_ORANGE: Color = Color::new(255, 165, 0);
const COLOR_GREEN: Color = Color::new(0, 255, 0);
const COLOR_OFF: Color = Color::new(0, 0, 0);

 pub struct SystemLed{
    thread: Thread,  
 }


 impl Initializable for SystemLed {
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        log_info!(APP_TAG, "Init app display");

        self.thread = self.thread.spawn_simple( move || {

            let rgb_led = RgbLed::new();
            rgb_led.set_color(&COLOR_OFF);
            
            

            loop {
                let status: u32 = StatusSignal::get().into();
                if status >= None.into() && status <= EnableWifi.into() {
                    if TIMER.load(Ordering::SeqCst) % (BLINK_INTERVAL_MS * 2) < BLINK_INTERVAL_MS {
                        rgb_led.set_color(&COLOR_ORANGE);
                    } else {
                        rgb_led.set_color(&COLOR_OFF);
                    }
                } else if (status & <StatusFlag as Into<u32>>::into(StatusFlag::Ready)) == Ready.into() {
                    Self::handle_ready(&rgb_led);
                } else if (status & <StatusFlag as Into<u32>>::into(StatusFlag::Error)) == Error.into() {
                    if TIMER.load(Ordering::SeqCst) % (BLINK_INTERVAL_MS * 2) < BLINK_INTERVAL_MS {
                        rgb_led.set_color(&COLOR_RED);
                    } else {
                        rgb_led.set_color(&COLOR_OFF);
                    }
                } else {
                   rgb_led.set_color(&COLOR_OFF);
                }

                // match status {
                //     None | Startup | EnableSystemHandler | EnableSession | EnableParser | EnableDisplay | EnableWifi => 
                //         if TIMER.load(Ordering::SeqCst) % (BLINK_INTERVAL_MS * 2) < BLINK_INTERVAL_MS {
                //             rgb_led.set_color(&COLOR_ORANGE);
                //         } else {
                //             rgb_led.set_color(&COLOR_OFF);
                //         },
                //     Ready => Self::handle_ready(&rgb_led),
                //     Error => {
                //         if TIMER.load(Ordering::SeqCst) % (BLINK_INTERVAL_MS * 2) < BLINK_INTERVAL_MS {
                //             rgb_led.set_color(&COLOR_RED);
                //         } else {
                //             rgb_led.set_color(&COLOR_OFF);
                //         }
                //     },
                //     _ => rgb_led.set_color(&COLOR_OFF),
                    
                // }
                
                
                System::delay_with_to_tick(Duration::from_millis(TICK_INTERVAL_MS as u64));
                TIMER.fetch_add(TICK_INTERVAL_MS as u16, Ordering::SeqCst);
            }
        })?;


        Ok(())
    }
 }

 impl SystemLed {
    pub fn new() -> Self {
        Self {
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal),
        }
    }

    fn handle_ready(rgb_led: &RgbLed) {
        rgb_led.set_color(&COLOR_GREEN);
    }
 }