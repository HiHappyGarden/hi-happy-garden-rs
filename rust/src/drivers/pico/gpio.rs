

mod ffi {
    #![allow(non_camel_case_types)]
    #![allow(dead_code)]

    #[repr(C)]
    pub struct pwm_config {
        pub csr: u32,
        pub div: u32,
        pub top: u32,
    }

    pub(super) const GPIO_OUT: bool = true;
    pub(super) const GPIO_IN: bool = false;  

    #[repr(u32)]
    #[derive(Clone, Copy)]
    pub(super) enum gpio_function_t {
        GPIO_FUNC_HSTX = 0,
        GPIO_FUNC_SPI = 1,
        GPIO_FUNC_UART = 2,
        GPIO_FUNC_I2C = 3,
        GPIO_FUNC_PWM = 4,
        GPIO_FUNC_SIO = 5,
        GPIO_FUNC_PIO0 = 6,
        GPIO_FUNC_PIO1 = 7,
        GPIO_FUNC_PIO2 = 8,
        GPIO_FUNC_GPCK = 9,
        // GPIO_FUNC_XIP_CS1 = 9,
        // GPIO_FUNC_CORESIGHT_TRACE = 9,
        GPIO_FUNC_USB = 10,
        GPIO_FUNC_UART_AUX = 11,
        GPIO_FUNC_NULL = 0x1f,
    }

    #[repr(u32)]
    #[derive(Clone, Copy)]
    pub(super) enum gpio_irq_level {
        GPIO_IRQ_LEVEL_LOW = 0x1,  
        GPIO_IRQ_LEVEL_HIGH = 0x2, 
        GPIO_IRQ_EDGE_FALL = 0x4,  
        GPIO_IRQ_EDGE_RISE = 0x8  
    }

    pub(super) const IO_IRQ_BANK0: u8 =  21;

    unsafe extern "C" {
        pub(super) fn hhg_gpio_init(gpio: u32);
        pub(super) fn hhg_gpio_set_dir(gpio: u32, out: bool);
        pub(super) fn hhg_gpio_put(gpio: u32, value: bool);
        pub(super) fn hhg_gpio_get(gpio: u32) -> bool;
        pub(super) fn hhg_gpio_pull_up(gpio: u32);
        pub(super) fn hhg_gpio_pull_down(gpio: u32);
        pub(super) fn hhg_gpio_disable_pulls(gpio: u32);
        pub(super) fn hhg_gpio_set_function(gpio: u32, fn_: u32);
        pub(super) fn hhg_pwm_gpio_to_slice_num(gpio: u32) -> u32;
        pub(super) fn hhg_pwm_get_default_config() -> pwm_config;
        pub(super) fn hhg_pwm_config_set_clkdiv(c: *mut pwm_config, div: f32);
        pub(super) fn hhg_pwm_init(slice_num: u32, c: *mut pwm_config, start: bool);
        pub(super) fn hhg_pwm_set_gpio_level(gpio: u32, level: u16);
        pub(super) fn hhg_gpio_set_irq_enabled_with_callback(gpio: u32, events: u32, enabled: bool, callback: extern "C" fn());
        pub(super) fn hhg_gpio_set_irq_enabled(gpio: u32, events: u32, enabled: bool);
        pub(super) fn hhd_irq_set_enabled(irq: u8, enabled: bool);
    }   
}


use core::default;
use core::str::FromStr;

use alloc::str;
use alloc::string::{String, ToString};

use osal_rs::os::config;
use osal_rs::{log_info, println};
use osal_rs::utils::{Error, OsalRsBool, Ptr, Result};

use crate::drivers::gpio::GpioConfigs;
use crate::drivers::pico::gpio::ffi::{GPIO_IN, IO_IRQ_BANK0, gpio_function_t, hhd_irq_set_enabled, hhg_gpio_get, hhg_gpio_init, hhg_gpio_pull_down, hhg_gpio_pull_up, hhg_gpio_put, hhg_gpio_set_dir, hhg_gpio_set_function, hhg_gpio_set_irq_enabled, hhg_gpio_set_irq_enabled_with_callback, hhg_pwm_config_set_clkdiv, hhg_pwm_get_default_config, hhg_pwm_gpio_to_slice_num, hhg_pwm_init, hhg_pwm_set_gpio_level};
use crate::drivers::gpio::{GpioFn, GpioConfig, GpioName, GpioInputType, InterruptCallback, InterruptConfig, InterruptType::{self, *}, GpioType};
use crate::traits::state::{Deinitializable, Initializable};
use GpioPeripheral::*;


pub const GPIO_CONFIG_SIZE: usize = 7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioPeripheral {
    NoUsed,
    EncoderA,
    EncoderB,
    EncoderBtn,
    Btn,
    LedRed,
    LedGreen,
    LedBlue,
}
 
impl GpioName for GpioPeripheral {
    fn as_str(&self) -> &str {
        match self {
            NoUsed => "NoUsed",
            EncoderA => "EncoderA",
            EncoderB => "EncoderB",
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
            "EncoderA" => Ok(EncoderA),
            "EncoderB" => Ok(EncoderB),
            "EncoderBtn" => Ok(EncoderBtn),
            "Btn" => Ok(Btn),
            "LedRed" => Ok(LedRed),
            "LedGreen" => Ok(LedGreen),
            "LedBlue" => Ok(LedBlue),
            _ => Err(Error::NotFound)
        }
    }
}

pub static GPIO_FN : GpioFn = GpioFn {
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

pub fn get_gpio_configs() -> GpioConfigs<'static, GPIO_CONFIG_SIZE> {
    GpioConfigs::new_with_array([
        Some(GpioConfig::new(&EncoderA, GpioType::Input(None, 21, GpioInputType::PullDown, 0))),
        Some(GpioConfig::new(&EncoderB, GpioType::Input(None, 20, GpioInputType::PullDown, 0))),
        Some(GpioConfig::new(&EncoderBtn, GpioType::Input(None, 19, GpioInputType::PullUp, 0))),
        Some(GpioConfig::new(&Btn, GpioType::Input(None, 19, GpioInputType::PullDown, 0))),
        Some(GpioConfig::new(&LedRed, GpioType::OutputPWM(None, 13, 0))),
        Some(GpioConfig::new(&LedGreen, GpioType::OutputPWM(None, 14, 0))),
        Some(GpioConfig::new(&LedBlue, GpioType::OutputPWM(None, 15, 0))),
    ])
}


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
    use ffi::gpio_irq_level::*;
    unsafe {
        match &irq_type {
            RisingEdge => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_EDGE_RISE as u32, enable, callback),
            FallingEdge => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_EDGE_FALL as u32, enable, callback),
            BothEdge => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_EDGE_RISE as u32 | GPIO_IRQ_EDGE_FALL as u32, enable, callback),
            HigthLevel => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_LEVEL_HIGH as u32, enable, callback),
            LowLevel => hhg_gpio_set_irq_enabled_with_callback(pin, GPIO_IRQ_LEVEL_LOW as u32, enable, callback),
        }
        hhd_irq_set_enabled(IO_IRQ_BANK0, true);
    }
    OsalRsBool::True
}

fn enable_interrupt(config: &GpioConfig, _: Option<Ptr>, pin: u32, enable: bool) -> OsalRsBool {
    use ffi::gpio_irq_level::*;

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
            HigthLevel => hhg_gpio_set_irq_enabled(pin, GPIO_IRQ_LEVEL_HIGH as u32, enable),
            LowLevel => hhg_gpio_set_irq_enabled(pin, GPIO_IRQ_LEVEL_LOW as u32, enable),
        }
    }
    OsalRsBool::True
}