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

use alloc::sync::Arc;

use osal_rs::log_info;
use osal_rs::os::{Mutex, Thread, ThreadFn};
use osal_rs::utils::Result;

use crate::drivers::platform::{self, Gpio, GpioPeripheral, GPIO_CONFIG_SIZE};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Encoder";

pub struct Encoder {
    gpio_ccw_ref: GpioPeripheral,
    gpio_cw_ref: GpioPeripheral,
    gpio_btn_ref: GpioPeripheral,
    gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>,
}

impl Encoder {
    pub fn new(gpio_ccw_ref: GpioPeripheral, gpio_cw_ref: GpioPeripheral, gpio_btn_ref: GpioPeripheral, gpio: Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Self {
        Self {
            gpio_ccw_ref,
            gpio_cw_ref,
            gpio_btn_ref,
            gpio,
        }
    }

    pub fn init(&mut self, _gpio: &mut Arc<Mutex<Gpio<GPIO_CONFIG_SIZE>>>) -> Result<()> {
        log_info!(APP_TAG, "Init encoder");

        Ok(())
    }
}
