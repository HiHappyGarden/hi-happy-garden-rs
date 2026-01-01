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
use core::time::Duration;

use alloc::str;
use alloc::sync::Arc;

use osal_rs::{log_error, log_info, print};
use osal_rs::os::{Mutex, MutexFn, System, SystemFn, Thread, ThreadFn};
use osal_rs::utils::{Error, OsalRsBool, Result};

use crate::drivers::gpio::InterruptType;
use crate::drivers::platform::{self, Gpio, GpioPeripheral, GPIO_CONFIG_SIZE};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Button";
const DEBOUNCE_TIME_US: u32 = 150_000; // Tempo di debounce in microsecondi (150ms)

static LAST_INTERRUPT_TIME: AtomicU32 = AtomicU32::new(0);

pub struct Button {
    gpio_ref: GpioPeripheral,
    gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>,
    thread: Thread,
}

extern "C" fn button_isr() {
    let current_time = System::get_current_time_us().as_micros() as u32;
    let last_time = LAST_INTERRUPT_TIME.load(Ordering::Relaxed);
    
    // Controlla se è passato abbastanza tempo dall'ultimo interrupt
    // Gestisce anche il wraparound del timer usando la sottrazione wrapping
    let elapsed = current_time.wrapping_sub(last_time);
    
    if elapsed >= DEBOUNCE_TIME_US {
        LAST_INTERRUPT_TIME.store(current_time, Ordering::Relaxed);
        log_info!(APP_TAG, "Button pressed (debounced)");
        
        // Qui puoi inserire la logica che vuoi eseguire al click del bottone
    }
}

impl Button {
    pub fn new(gpio_ref: GpioPeripheral, gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Self {
        Self {
            gpio_ref,
            gpio,
            thread: Thread::new("button_trd", 1024, 3)
        }
    }
    
    pub fn init(&mut self, gpio: &mut Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Result<()> {
        log_info!(APP_TAG, "Init button");


        if gpio.lock()?.set_interrupt(&self.gpio_ref, InterruptType::RisingEdge, true, button_isr) == OsalRsBool::False {
            log_error!(APP_TAG, "Error setting button interrupt");
            return Err(Error::NotFound);
        }


        // let gpio_clone = Arc::clone(gpio);

        // self.thread.spawn_simple(move || {

            
        //     loop {
        //         match gpio_clone.lock().unwrap().read(&GpioPeripheral::Btn) {
        //             Ok(value) => log_info!(APP_TAG, "Button:{}", value),
        //             Err(_) => log_error!(APP_TAG, "Error reading button"),
        //         }

        //         System::delay_with_to_tick(Duration::from_millis(500u64));
        //     }

        // })?;

        Ok(())
    }
}

