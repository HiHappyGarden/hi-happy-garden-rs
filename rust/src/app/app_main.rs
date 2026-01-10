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


use core::ptr::addr_of_mut;

use osal_rs::log_info;
use osal_rs::os::{System, SystemFn};
use osal_rs::utils::Result;
use crate::app::lcd::Lcd;
use crate::drivers::platform::Hardware;
use crate::traits::hardware::HardwareFn;
use crate::traits::state::Initializable;

const APP_TAG: &str = "AppMain";

static mut LCD : Lcd = Lcd::new();


pub struct AppMain{
    hardware: &'static mut Hardware
}

impl Initializable for AppMain {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app main");

        unsafe { 
            let lcd = &mut *addr_of_mut!(LCD);
            lcd.init()?;
            
            self.hardware.set_button_handler(lcd);
            self.hardware.set_encoder_handler(lcd);
              
        }
        
        log_info!(APP_TAG, "App main initialized successfully heap_free:{}", System::get_free_heap_size());
        Ok(())
    }
}

impl AppMain {
    pub fn new(hardware: &'static mut Hardware) -> Self {
        Self {
            hardware
        }
    }
}