/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
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

//! I2C bus tests: both buses answer and the expected devices are wired.
//!
//! `init` only reconfigures the same pins and baudrate `Hardware::init`
//! already set, so it is safe to call again on a running board.

use osal_rs::log_debug;
use osal_rs::utils::Result;

use super::I2C;
use crate::drivers::platform::{I2C0_INSTANCE, I2C1_INSTANCE, I2C_BAUDRATE, LCDDisplay};
use crate::drivers::plt::rtc_ds3231::RTC_DS3231_I2C_ADDRESS;
use crate::tests::{TestStats, run_tests, test_assert};
use crate::traits::state::Initializable;

const TAG: &str = "I2CTests";

fn test_bus0_has_rtc() -> Result<()> {
    let mut i2c = I2C::<{I2C0_INSTANCE}, {I2C_BAUDRATE}>::new();
    i2c.init()?;
    let devices = i2c.scan()?;
    log_debug!(TAG, "I2C0 devices: {:02x?}", devices);
    test_assert!(devices.contains(&RTC_DS3231_I2C_ADDRESS), "DS3231 not found on I2C0: {devices:02x?}");
    Ok(())
}

fn test_bus1_has_display() -> Result<()> {
    let mut i2c = I2C::<{I2C1_INSTANCE}, {I2C_BAUDRATE}>::new();
    i2c.init()?;
    let devices = i2c.scan()?;
    log_debug!(TAG, "I2C1 devices: {:02x?}", devices);
    test_assert!(devices.contains(&LCDDisplay::I2C_ADDRESS), "SH1106 not found on I2C1: {devices:02x?}");
    Ok(())
}

fn test_absent_device_nacks() -> Result<()> {
    // 0x08 is reserved-adjacent and unused on this board: a read must fail, not hang
    let mut i2c = I2C::<{I2C0_INSTANCE}, {I2C_BAUDRATE}>::new_with_address(0x08);
    i2c.init()?;
    let mut buffer = [0u8; 1];
    test_assert!(i2c.read(&mut buffer).is_err());
    Ok(())
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_bus0_has_rtc,
        test_bus1_has_display,
        test_absent_device_nacks,
    );
}
