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
use osal_rs::utils::OsalRsBool;

use crate::drivers::gpio::Gpio;
use crate::drivers::platform::{GPIO_CONFIG_SIZE, GpioPeripheral};
use crate::traits::relays::Relays as RaleayFn;
use crate::traits::state::Initializable;

const APP_TAG: &str = "Relays";

pub struct Relays (Gpio<GPIO_CONFIG_SIZE>);


impl Initializable for Relays {
    fn init(&mut self) -> osal_rs::utils::Result<()> {
        log_info!(APP_TAG, "Init relays");

        // Turn off all relays at startup
        self.turn_off_all_relays();

        Ok(())

    }
}


impl RaleayFn for Relays {
    fn set_relay_state(&self, relay_index: GpioPeripheral, state: bool) -> OsalRsBool {

        use GpioPeripheral::*;

        match relay_index {
            Relay1 | Relay2 | Relay3 | Relay4 => {
                self.0.get_mutex().lock();
                self.0.write(&relay_index, if state { 1 } else { 0 });
                self.0.unlock();
                OsalRsBool::True
            }
            _ => OsalRsBool::False,
        }

    }
}

impl Relays {
    pub fn new() -> Self {
        Self (Gpio::new())    
    }
}
