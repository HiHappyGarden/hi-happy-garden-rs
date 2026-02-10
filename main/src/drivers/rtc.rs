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

use crate::drivers::i2c::{I2C, I2CFn};
use crate::drivers::platform::{I2C0_INSTANCE, I2C_BAUDRATE};
use crate::traits::state::Initializable;

const APP_TAG: &str = "RTC";

pub struct RTC (I2C<{RTC::I2C_ADDRESS}, {I2C0_INSTANCE}>);

impl Initializable for RTC {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init RTC");

        
        Ok(())
    }
}

impl RTC {
    pub const I2C_ADDRESS: u8 = 0x68;

    pub fn new(i2c: I2C<{RTC::I2C_ADDRESS}, {I2C0_INSTANCE}>) -> Self {
        Self (i2c)
    }

}