/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

use osal_rs::log_info;
use osal_rs::os::{System, SystemFn};
use osal_rs::utils::Result;

use crate::apps::config::Config;
use crate::apps::display::{Display};
use crate::apps::parser::Parser;
use crate::apps::signals::error::ErrorSignal;
use crate::apps::wifi::WifiApp;
use crate::drivers::platform::{Hardware, LCDDisplay};
use crate::traits::hardware::HardwareFn;
use crate::traits::rx_tx::SetOnReceive;
use crate::traits::state::Initializable;
use crate::traits::wifi::SetOnWifiChangeStatus;

const APP_TAG: &str = "AppMain";

pub struct AppMain {
    hardware: &'static mut Hardware,
    config: &'static mut Config,
    display: Display<LCDDisplay>,
    wifi: WifiApp<'static>,
    parser: Parser,
}


impl Initializable for AppMain{
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app main");

        ErrorSignal::init()?;


        self.config.init()?;
        self.parser.init()?;
        self.wifi.init()?;
        self.display.init()?;
        self.display.set_enabled_wifi(self.config.get_wifi_config().is_enabled());

        // SAFETY: AppMain lives in static mut APP_MAIN, initialized once at startup.
        // We use raw pointers to avoid borrow checker issues, then convert to 'static refs.
        unsafe {
            let display_ptr = &raw const self.display;
            let wifi_ptr = &raw mut self.wifi;
            let config_ptr = &raw const self.config;
            let parser_ptr = &raw const self.parser;
            let hardware_ptr = &raw mut self.hardware;
            
            

            // Set RTC for wifi
            (*wifi_ptr).set_rtc((*hardware_ptr).get_rtc());
            
            // Set hardware callbacks - convert raw pointers to 'static references
            (*hardware_ptr).set_button_handler(&*display_ptr);
            (*hardware_ptr).set_encoder_handler(&*display_ptr);            

            // Set wifi configuration
            (*wifi_ptr).set_ntp_config(&*config_ptr);
            (*hardware_ptr).set_on_wifi_change_status(&mut *wifi_ptr);
            (*hardware_ptr).set_on_receive(&*parser_ptr);

        }

//test funzionalità

        // self.hardware.set_color(0, 0, 255); // Blue

        // self.hardware.set_relay_state(GpioPeripheral::Relay1, true);

        // let unique_id = Hardware::get_unique_id();
        // log_info!(APP_TAG, "Device Unique ID: {:02X?}", unique_id);

        log_info!(APP_TAG, "App main initialized successfully heap_free:{}", System::get_free_heap_size());

        Ok(())
    }
}

impl AppMain {
    pub fn new(hardware: &'static mut Hardware) -> Self {

        let display = Display::new(hardware.get_rtc(), hardware.get_lcd_display());
        
        Self {
            hardware,
            config: Config::new(),
            display,
            wifi: WifiApp::new(),
            parser: Parser::new(),
        }
    }
}