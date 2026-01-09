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


use osal_rs::{arcmux, log_info};
use osal_rs::os::{MutexFn, System, SystemFn};
use osal_rs::utils::{ArcMux, Result};
use crate::app::lcd::Lcd;
use crate::drivers::platform::Hardware;
use crate::traits::hardware::HardwareFn;
use crate::traits::state::Initializable;

const APP_TAG: &str = "AppMain";

pub struct AppMain<'a> {
    hardware: &'a mut Hardware,
    lcd: ArcMux<Lcd>
}

impl Initializable for AppMain<'_> {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app main");

        ArcMux::clone(&self.lcd).lock().unwrap().init()?;

        let lcd = ArcMux::clone(&self.lcd);
        self.hardware.set_button_handler(lcd);

        let lcd = ArcMux::clone(&self.lcd);
        self.hardware.set_encoder_handler(lcd);
        
        log_info!(APP_TAG, "App main initialized successfully heap_free:{}", System::get_free_heap_size());
        Ok(())
    }
}

impl<'a> AppMain<'a> {
    pub fn new(hardware: &'a mut Hardware) -> Self {

        AppMain {
            hardware,
            lcd: arcmux!(Lcd::new())
        }
    }
}