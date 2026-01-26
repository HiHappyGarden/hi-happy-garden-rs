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
use osal_rs::utils::{Error, Result};

use crate::apps::display::{Display};
use crate::drivers::platform::{GpioPeripheral, Hardware, LCDDisplay};
use crate::traits::hardware::HardwareFn;
use crate::traits::relays::Relays;
use crate::traits::rgb_led::RgbLed;
use crate::traits::state::Initializable;

const APP_TAG: &str = "AppMain";

#[derive(osal_rs_serde::Serialize, osal_rs_serde::Deserialize, Debug, Default)]
struct Test {
    a: u8,
    b: bool,
    c: u16,
    // Changed from &'static str to String for serialization support
    s: alloc::string::String,
}

#[derive(osal_rs_serde::Serialize, osal_rs_serde::Deserialize, Debug, Default)]
struct Test2 {
    test: Test,
    pippo: f32,
}


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


        let test = Test {
            a: 42,
            b: true,
            c: 65535,
            s: alloc::string::String::from("Hello, World!"),
        };

        let test2 = Test2 {
            test,
            pippo: 3.14,
        };

        let json_str = cjson_binding::to_json(&test2).map_err(|_| Error::Unhandled("Serialization error"))?;

        log_info!(APP_TAG, "Serialized JSON: {}", json_str);


        let deserializer = cjson_binding::from_json::<Test2>(&json_str).map_err(|_| Error::Unhandled("Deserialization error"))?;

        log_info!(APP_TAG, "Deserialized JSON: {:?}", deserializer);

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