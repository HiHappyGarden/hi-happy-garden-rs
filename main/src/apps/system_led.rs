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

use core::time::Duration;

use osal_rs::log_info;
use osal_rs::os::{System, Thread, ThreadFn};
use osal_rs::os::types::StackType;

use crate::drivers::platform::ThreadPriority;
use crate::drivers::rgb_led::RgbLed;
use crate::traits::rgb_led::RgbLed as _;
use crate::traits::state::Initializable;



enum Status {
    Initializing,
    Running,
}

const APP_TAG: &str = "AppSystemLed";
const THREAD_NAME: &str = "system_led_trd";
const STACK_SIZE: StackType = 256;
const BLINK_INTERVAL_MS: u64 = 500;
const DELAY_IN_MS: u64 = 100; 


static mut STATUS: Status = Status::Initializing;

 pub struct SystemLed(Thread);


 impl Initializable for SystemLed {
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        log_info!(APP_TAG, "Init app display");

        
        self.0 = self.0.spawn_simple( move || {

            let rgb_led = RgbLed::new();
            rgb_led.set_color(0, 0, 0);
            
            let mut timer = 0;

            loop {
                unsafe {
                    match STATUS {
                        Status::Initializing => {

                            if timer <= BLINK_INTERVAL_MS {
                                rgb_led.set_color(255, 165, 0); // Orange
                            } else if timer <= BLINK_INTERVAL_MS * 2 {
                                rgb_led.set_color(0, 0, 0); // Off
                            } else {
                                timer = 0;
                            }
                            
                        },
                        Status::Running => {
                            rgb_led.set_color(0, 255, 0); // Green
                        },
                    }
                }
                
                System::delay_with_to_tick(Duration::from_millis(DELAY_IN_MS));
                timer += DELAY_IN_MS;
            }
        })?;


        Ok(())
    }
 }

 impl SystemLed {
    pub fn new() -> Self {
        Self(Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::Normal))
    }

    #[allow(unused)]
    pub fn set_running() {
         unsafe {
            STATUS = Status::Running;
        }
    }
 }