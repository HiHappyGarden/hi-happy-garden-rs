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
use crate::app::lcd::{Lcd};
use crate::drivers::platform::{GpioPeripheral, Hardware};
use crate::traits::hardware::HardwareFn;
use crate::traits::lcd_display::LCDDisplay;
use crate::traits::relays::Relays;
use crate::traits::rgb_led::RgbLed;
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

        
        
        self.hardware.set_color(0, 0, 255); // Blue

        self.hardware.set_relay_state(GpioPeripheral::Relay1, true);

        self.hardware.set_internal_led(true);

        unsafe {
            let lcd_display = self.hardware.get_lcd_display();
            let lcd = &mut *addr_of_mut!(LCD);
            // Cast the lifetime to 'static since both LCD and hardware are 'static
            lcd.set_display(core::mem::transmute::<&mut dyn LCDDisplay, &'static mut dyn LCDDisplay>(lcd_display));
        }
        

        // lcd_display.draw_rect(10, 0, 3, 3, LCDWriteMode::ADD)?;

        // lcd_display.draw()?;

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