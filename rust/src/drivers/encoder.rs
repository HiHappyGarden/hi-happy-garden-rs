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
use osal_rs::log_info;
use osal_rs::os::config::MINIMAL_STACK_SIZE;
use osal_rs::os::types::StackType;
use osal_rs::os::{EventGroup, EventGroupFn, Mutex, MutexFn, System, SystemFn, Thread, ThreadFn, ThreadParam};
use osal_rs::utils::Result;

use crate::drivers::button::Button;
use crate::drivers::button::button_events::*;
use crate::drivers::gpio::{InterruptType};
use crate::drivers::platform::{self, GPIO_CONFIG_SIZE, Gpio, GpioPeripheral, OsalThreadPriority};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Encoder";
const APP_THREAD_NAME: &str = "encoder_trd";
const APP_THREAD_STACK: StackType = MINIMAL_STACK_SIZE as StackType;

static EVENT_HANDLER: OnceBox<Arc<EventGroup>> = OnceBox::new();
static BUTTON_STATE: AtomicU32 = AtomicU32::new(0);

#[allow(dead_code)]
pub struct Encoder {
    gpio_ccw_ref: GpioPeripheral,
    gpio_cw_ref: GpioPeripheral,
    btn: Button,
    thread: Thread,
    
}

extern "C" fn encoder_button_isr() {
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


impl Encoder {
    pub fn new(gpio_ccw_ref: GpioPeripheral, gpio_cw_ref: GpioPeripheral, gpio_btn_ref: GpioPeripheral, gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Self {

        let event_handler = EVENT_HANDLER.get_or_init(|| Box::new(Arc::new(EventGroup::new().unwrap())));

        Self {
            gpio_ccw_ref,
            gpio_cw_ref,
            btn: Button::new_with_external_callback(gpio_btn_ref, gpio, encoder_button_isr, Some(Arc::clone(event_handler) as ThreadParam) ),
            thread: Thread::new_with_to_priority(APP_THREAD_NAME, APP_THREAD_STACK, OsalThreadPriority::Normal)
        }
    }

    pub fn init(&mut self, gpio: &mut Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Result<()> {
        log_info!(APP_TAG, "Init encoder");

        self.btn.init(gpio)?;


        let gpio_clone = Arc::clone(&gpio);
        let gpio_cw_ref = self.gpio_cw_ref.clone();
        let gpio_ccw_ref = self.gpio_ccw_ref.clone();
        self.thread.spawn_simple(move || {

            log_info!(APP_TAG, "Encoder thread started");


            loop {
                
                let cw = gpio_clone.lock().unwrap().read(&gpio_cw_ref).unwrap_or(0u32);
                let ccw = gpio_clone.lock().unwrap().read(&gpio_ccw_ref).unwrap_or(0u32);

                log_info!(APP_TAG, "Encoder state: CW={}, CCW={}", cw, ccw);

                System::delay(100);
            }


        })?;


        Ok(())
    }
}
