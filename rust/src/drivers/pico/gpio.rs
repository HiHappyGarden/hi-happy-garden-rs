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

use core::default;
use core::str::FromStr;

use alloc::str;
use alloc::string::{String, ToString};

use osal_rs::os::config;
use osal_rs::{log_info, println};
use osal_rs::utils::{AsSyncStr, Error, OsalRsBool, Ptr, Result};

use crate::drivers::gpio::GpioConfigs;
use crate::drivers::pico::ffi::{GPIO_IN, gpio_function_t, hhg_gpio_get, hhg_gpio_init, hhg_gpio_pull_down, hhg_gpio_pull_up, hhg_gpio_put, hhg_gpio_set_dir, hhg_gpio_set_function, hhg_gpio_set_irq_enabled, hhg_gpio_set_irq_enabled_with_callback, hhg_pwm_config_set_clkdiv, hhg_pwm_get_default_config, hhg_pwm_gpio_to_slice_num, hhg_pwm_init, hhg_pwm_set_gpio_level};
use crate::drivers::gpio::{GpioFn, GpioConfig, GpioInputType, InterruptCallback, InterruptConfig, InterruptType::{self, *}, GpioType};
use crate::traits::state::{Deinitializable, Initializable};
use GpioPeripheral::*;


pub const GPIO_CONFIG_SIZE: usize = 7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioPeripheral {
    NoUsed,
    EncoderCCW,
    EncoderCW,
    EncoderBtn,
    Btn,
    LedRed,
    LedGreen,
    LedBlue,
}
 
impl AsSyncStr for GpioPeripheral {
    fn as_str(&self) -> &str {
        match self {
            NoUsed => "NoUsed",
            EncoderCCW => "EncoderCCw",
            EncoderCW => "EncoderCW",
            EncoderBtn => "EncoderBtn",
            Btn => "Btn",
            LedRed => "LedRed",
            LedGreen => "LedGreen",
            LedBlue => "LedBlue",
        }
    }
}

impl FromStr for GpioPeripheral {
    type Err = Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "NoUsed" => Ok(NoUsed),
            "EncoderCCw" => Ok(EncoderCCW),
            "EncoderCW" => Ok(EncoderCW),
            "EncoderBtn" => Ok(EncoderBtn),
            "Btn" => Ok(Btn),
            "LedRed" => Ok(LedRed),
            "LedGreen" => Ok(LedGreen),
            "LedBlue" => Ok(LedBlue),
            _ => Err(Error::NotFound)
        }
    }
}

pub(super) fn get_gpio_configs() -> GpioConfigs<'static, GPIO_CONFIG_SIZE> {
    GpioConfigs::new_with_array([
        Some(GpioConfig::new(&EncoderCCW, GpioType::Input(None, 20, GpioInputType::PullDown, 0))),
        Some(GpioConfig::new(&EncoderCW, GpioType::Input(None, 21, GpioInputType::PullDown, 0))),
        Some(GpioConfig::new(&EncoderBtn, GpioType::Input(None, 19, GpioInputType::PullUp, 0))),
        Some(GpioConfig::new(&Btn, GpioType::Input(None, 18, GpioInputType::PullUp, 0))),
        Some(GpioConfig::new(&LedRed, GpioType::OutputPWM(None, 13, 0))),
        Some(GpioConfig::new(&LedGreen, GpioType::OutputPWM(None, 14, 0))),
        Some(GpioConfig::new(&LedBlue, GpioType::OutputPWM(None, 15, 0))),
    ])
}

pub(super) const GPIO_FN : GpioFn = GpioFn {
    init: None,
    input: Some(input),
    input_analog: None,
    output: None,
    output_pwm: Some(output_pwm),
    peripheral: None,
    deinit: None,
    read: Some(read),
    write: Some(write),
    set_pwm: Some(set_pwm),
    set_interrupt: Some(set_interrupt),
    enable_interrupt: Some(enable_interrupt)
};

fn input(_: &GpioConfig, _: Option<Ptr>, pin: u32, input_type: GpioInputType, default_value: u32) -> Result<()> {

    unsafe {
        hhg_gpio_init(pin);   
        hhg_gpio_set_dir(pin, GPIO_IN);

        use GpioInputType::*;
        match input_type {
            NoPull => (),
            PullUp => hhg_gpio_pull_up(pin),
            PullDown => hhg_gpio_pull_down(pin),
        }

        hhg_gpio_put(pin, default_value != 0);
    }

    Ok(())
}

fn output_pwm(_: &GpioConfig, _: Option<Ptr>, pin: u32, default_value: u32) -> Result<()> {

    unsafe {
        hhg_gpio_set_function(pin, gpio_function_t::GPIO_FUNC_PWM as u32);
        let slice_num = hhg_pwm_gpio_to_slice_num(pin);
        let mut pwm_config = hhg_pwm_get_default_config();
        hhg_pwm_config_set_clkdiv(&mut pwm_config, 4.0);
        hhg_pwm_init(slice_num, &mut pwm_config, false);
        hhg_pwm_set_gpio_level(pin, default_value as u16);
    }

    Ok(())
}

fn read(_: &GpioConfig, _: Option<Ptr>, pin: u32) -> Result<u32> {
    let value = unsafe {hhg_gpio_get(pin)};
    Ok(value as u32)    
}

fn write(_: &GpioConfig, _: Option<Ptr>, pin: u32, state: u32) -> OsalRsBool {
    unsafe {
        hhg_gpio_put(pin, state != 0);
    }
    OsalRsBool::True
}

fn set_pwm(_: &GpioConfig, _: Option<Ptr>, pin: u32, value: u32) -> OsalRsBool {
    unsafe {
        hhg_pwm_set_gpio_level(pin, value as u16);
    }
    OsalRsBool::True
}

fn set_interrupt(_: &GpioConfig, _: Option<Ptr>, pin: u32, irq_type: InterruptType, callback: InterruptCallback, enable: bool) -> OsalRsBool { 
    use super::ffi::gpio_irq_level::*;
    unsafe {
        match &irq_type {
            RisingEdge => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_EDGE_RISE as u32, enable, callback),
            FallingEdge => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_EDGE_FALL as u32, enable, callback),
            BothEdge => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_EDGE_RISE as u32 | GPIO_IRQ_EDGE_FALL as u32, enable, callback),
            HighLevel => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_LEVEL_HIGH as u32, enable, callback),
            LowLevel => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_LEVEL_LOW as u32, enable, callback),
        }
    }
    OsalRsBool::True
}

fn enable_interrupt(config: &GpioConfig, _: Option<Ptr>, pin: u32, enable: bool) -> OsalRsBool {
    use super::ffi::gpio_irq_level::*;

    let InterruptConfig{irq_type, ..}  = if let Some(irq_config) = &config.irq {
        irq_config.clone()
    } else {
        return OsalRsBool::False
    };

    unsafe {
        match &irq_type {
            RisingEdge => hhg_gpio_set_irq_enabled(pin, GPIO_IRQ_EDGE_RISE as u32, enable),
            FallingEdge => hhg_gpio_set_irq_enabled(pin, GPIO_IRQ_EDGE_FALL as u32, enable),
            BothEdge => hhg_gpio_set_irq_enabled(pin, GPIO_IRQ_EDGE_RISE as u32 | GPIO_IRQ_EDGE_FALL as u32, enable),
            HighLevel => hhg_gpio_set_irq_enabled(pin, GPIO_IRQ_LEVEL_HIGH as u32, enable),
            LowLevel => hhg_gpio_set_irq_enabled(pin, GPIO_IRQ_LEVEL_LOW as u32, enable),
        }
    }
    OsalRsBool::True
}