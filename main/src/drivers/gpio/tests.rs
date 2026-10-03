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


//! GPIO tests on the board pin map: inputs read their idle level, the ADC
//! answers and every configured output accepts writes.
//!
//! Nobody must touch the button or the encoder while these run.

use core::time::Duration;

use osal_rs::log_debug;
use osal_rs::os::System;
use osal_rs::utils::{Error, OsalRsBool, Result};

use super::Gpio;
use crate::drivers::platform::GpioPeripheral;
use crate::tests::{TestStats, hardware, run_tests, test_assert, test_assert_eq};
use crate::traits::hardware::HardwareFn;

const TAG: &str = "GpioTests";

fn test_buttons_idle() -> Result<()> {
    let gpio = Gpio::shared();
    // Pull-up inputs, active low: released reads 1
    test_assert_eq!(gpio.read(&GpioPeripheral::Btn)?, 1);
    test_assert_eq!(gpio.read(&GpioPeripheral::EncoderBtn)?, 1);
    // Encoder phases rest on either level depending on the detent
    test_assert!(gpio.read(&GpioPeripheral::EncoderCW)? <= 1);
    test_assert!(gpio.read(&GpioPeripheral::EncoderCCW)? <= 1);
    Ok(())
}

fn test_adc_range() -> Result<()> {
    let gpio = Gpio::shared();
    for _ in 0..10 {
        let raw = gpio.read(&GpioPeripheral::InternalTemp)?;
        test_assert!(raw > 0 && raw < 4_096, "ADC out of 12-bit range: {raw}");
    }
    Ok(())
}

fn test_temperature() -> Result<()> {
    let temperature = hardware().get_temperature();
    log_debug!(TAG, "chip temperature: {temperature:.1} C");
    test_assert!(temperature > 0.0 && temperature < 85.0, "implausible chip temperature: {temperature}");
    Ok(())
}

fn test_temperature_conversion() -> Result<()> {
    // 0.706 V is 27 C by the RP2350 datasheet formula
    let raw_27c = (0.706f32 / 3.3f32 * 4_096f32) as u32;
    let temperature = <crate::drivers::platform::Hardware as HardwareFn>::temperature_conversion(raw_27c);
    test_assert!((temperature - 27.0).abs() < 1.0, "27 C converted to {temperature}");
    Ok(())
}

fn test_wrong_io_type() -> Result<()> {
    let gpio = Gpio::shared();
    // Writes on inputs and reads on outputs are refused, not forwarded to the pin
    test_assert_eq!(gpio.write(&GpioPeripheral::Btn, 1), OsalRsBool::False);
    test_assert!(matches!(gpio.read(&GpioPeripheral::LedRed), Err(Error::InvalidType)));
    test_assert_eq!(gpio.set_pwm(&GpioPeripheral::Relay0, 10), OsalRsBool::False);
    // NoUsed has no entry in the pin map
    test_assert!(matches!(gpio.read(&GpioPeripheral::NoUsed), Err(Error::NotFound)));
    test_assert_eq!(gpio.write(&GpioPeripheral::NoUsed, 1), OsalRsBool::False);
    Ok(())
}

fn test_rgb_led_pwm() -> Result<()> {
    let gpio = Gpio::shared();
    let leds = [GpioPeripheral::LedRed, GpioPeripheral::LedGreen, GpioPeripheral::LedBlue];
    for led in leds {
        test_assert_eq!(gpio.set_pwm(&led, 255), OsalRsBool::True);
        System::delay_with_to_tick(Duration::from_millis(150));
        test_assert_eq!(gpio.set_pwm(&led, 0), OsalRsBool::True);
    }
    Ok(())
}

fn test_cyw43_led() -> Result<()> {
    let gpio = Gpio::shared();
    test_assert_eq!(gpio.write(&GpioPeripheral::Cyw43Led, 1), OsalRsBool::True);
    System::delay_with_to_tick(Duration::from_millis(150));
    test_assert_eq!(gpio.write(&GpioPeripheral::Cyw43Led, 0), OsalRsBool::True);
    Ok(())
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_buttons_idle,
        test_adc_range,
        test_temperature_conversion,
        test_temperature,
        test_wrong_io_type,
        test_rgb_led_pwm,
        test_cyw43_led,
    );
}
