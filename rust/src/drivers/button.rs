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
use core::time::Duration;

use alloc::str;
use alloc::sync::Arc;

use osal_rs::{log_error, log_info, print};
use osal_rs::os::{Mutex, MutexFn, System, SystemFn, Thread, ThreadFn};
use osal_rs::utils::Result;

use crate::drivers::platform::{self, Gpio, GpioPeripheral, GPIO_CONFIG_SIZE};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Button";

pub struct Button {
    gpio_ref: GpioPeripheral,
    gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>,
    thread: Thread,
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

