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
use osal_rs::{log_info, minimal_stack_size};
use osal_rs::os::types::StackType;
use osal_rs::os::{EventGroup, EventGroupFn, Mutex, MutexFn, System, SystemFn, Thread, ThreadFn, ThreadParam};
use osal_rs::utils::Result;

use crate::drivers::button::Button;
use crate::drivers::button::button_events::*;
use crate::drivers::gpio::{InterruptType};
use crate::drivers::platform::{self, GPIO_CONFIG_SIZE, Gpio, GpioPeripheral, OsalThreadPriority};
use crate::traits::state::Initializable;
use encoder_events::*;

const APP_TAG: &str = "Encoder";
const APP_THREAD_NAME: &str = "encoder_trd";

static EVENT_HANDLER: OnceBox<Arc<EventGroup>> = OnceBox::new();
static BUTTON_STATE: AtomicU32 = AtomicU32::new(0);

pub mod encoder_events {
    use osal_rs::os::types::EventBits;

    pub const ENCODER_NONE: EventBits = 0x0000;
    pub const ENCODER_CCW_RISE: EventBits = 0x0004;
    pub const ENCODER_CCW_FALL: EventBits = 0x0008;
    pub const ENCODER_CW_RISE: EventBits = 0x0010;
    pub const ENCODER_CW_FALL: EventBits = 0x0020;
}


#[allow(dead_code)]
pub struct Encoder {
    gpio_ccw_ref: GpioPeripheral,
    gpio_cw_ref: GpioPeripheral,
    gpio_btn_ref: GpioPeripheral,
    thread: Thread,
    
}

extern "C" fn encoder_button_isr() {

}

extern "C" fn encoder_ccw_isr() {

}

extern "C" fn encoder_cw_isr() {

}


impl Encoder {
    pub fn new(gpio_ccw_ref: GpioPeripheral, gpio_cw_ref: GpioPeripheral, gpio_btn_ref: GpioPeripheral, gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Self {
        let event_handler = EVENT_HANDLER.get_or_init(|| Box::new(Arc::new(EventGroup::new().unwrap())));

        Self {
            gpio_ccw_ref,
            gpio_cw_ref,
            gpio_btn_ref,
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, minimal_stack_size!(), OsalThreadPriority::Normal)
        }
    }

    pub fn init(&mut self, gpio: &mut Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Result<()> {
        log_info!(APP_TAG, "Init encoder");





        
        self.thread.spawn_simple(move || {

            log_info!(APP_TAG, "Encoder thread started");


            loop {
                



                System::delay(100);
            }


        })?;



        Ok(())
    }
}
