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

use osal_rs::log_info;
use osal_rs::os::{System, SystemFn};
use osal_rs::utils::Result;

use crate::apps::display::{Display};
use crate::drivers::platform::{GpioPeripheral, Hardware, LCDDisplay};
use crate::traits::hardware::HardwareFn;
use crate::traits::relays::Relays;
use crate::traits::rgb_led::RgbLed;
use crate::traits::state::Initializable;

const APP_TAG: &str = "AppMain";




pub struct AppMain{
    hardware: &'static mut Hardware,
    display: Display<LCDDisplay>,
}


impl Initializable for AppMain {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app main");

        
        
        self.display.init()?;
        
        // SAFETY: AppMain has 'static lifetime since it's created with 'static hardware
        let display_ref: &'static Display<LCDDisplay>  = unsafe { &*(&self.display as *const _) };
        
        self.hardware.set_button_handler(display_ref);
        
        self.hardware.set_encoder_handler(display_ref);
    

//test funzionalità

        self.hardware.set_color(0, 0, 255); // Blue

        self.hardware.set_relay_state(GpioPeripheral::Relay1, true);

        self.hardware.set_internal_led(true);

        self.display.draw()?;


        log_info!(APP_TAG, "App main initialized successfully heap_free:{}", System::get_free_heap_size());

        Ok(())
    }
}

impl AppMain {
    pub fn new(hardware: &'static mut Hardware) -> Self {
        let display = Display::new(hardware.get_lcd_display());
        Self {
            hardware,
            display,
        }
    }
}