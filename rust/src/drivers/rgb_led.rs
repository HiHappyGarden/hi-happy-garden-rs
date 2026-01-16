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
use osal_rs::utils::Result;

use crate::drivers::gpio::Gpio;
use crate::drivers::platform::{GpioPeripheral, GPIO_CONFIG_SIZE};
use crate::traits::rgb_led::RgbLed as RgbLedFn;
use crate::traits::state::Initializable;


const APP_TAG: &str = "RgbLed";

 pub struct RgbLed {
    gpio_red_ref: GpioPeripheral,
    gpio_green_ref: GpioPeripheral,
    gpio_blue_ref: GpioPeripheral,
    gpio: Gpio<GPIO_CONFIG_SIZE>,
}

impl Initializable for RgbLed {
    fn init(&mut self) -> Result<()> {
        
        log_info!(APP_TAG, "Init RGB LED");

         // Initialize the GPIOs for RGB LED    
        self.turn_off();
        Ok(())

    }
}

impl RgbLedFn for RgbLed {
    fn set_color(&self, red: u8, green: u8, blue: u8) {
        // Set the red LED
        self.set_red(red);
        
        // Set the green LED
        self.set_green(green);

        // Set the blue LED
        self.set_blue(blue);
    }

    fn  set_red(&self, red: u8) {
        self.gpio.get_mutex().lock();
        self.gpio.set_pwm(&self.gpio_red_ref, red as u16);
        self.gpio.get_mutex().unlock();
    }

    fn set_green(&self, green: u8) {
        self.gpio.get_mutex().lock();
        self.gpio.set_pwm(&self.gpio_green_ref, green as u16);
        self.gpio.get_mutex().unlock();
    }

    fn set_blue(&self, blue: u8) {
        self.gpio.get_mutex().lock();
        self.gpio.set_pwm(&self.gpio_blue_ref, blue as u16);
        self.gpio.get_mutex().unlock();
    }
}

impl RgbLed {
    pub fn new() -> Self {
        RgbLed {
            gpio_red_ref: GpioPeripheral::LedRed,
            gpio_green_ref: GpioPeripheral::LedGreen,
            gpio_blue_ref: GpioPeripheral::LedBlue,
            gpio: Gpio::new(),
        }
    }
}

