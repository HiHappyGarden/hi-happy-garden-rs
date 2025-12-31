

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


use core::str::FromStr;

use alloc::string::{String, ToString};
use osal_rs::{log_info, println};
use osal_rs::utils::{Error, Result, OsalRsBool};
use crate::drivers::pico::gpio::ffi::{GPIO_IN, IO_IRQ_BANK0, gpio_function_t, hhd_irq_set_enabled, hhg_gpio_get, hhg_gpio_init, hhg_gpio_pull_down, hhg_gpio_pull_up, hhg_gpio_put, hhg_gpio_set_dir, hhg_gpio_set_function, hhg_gpio_set_irq_enabled, hhg_gpio_set_irq_enabled_with_callback, hhg_pwm_config_set_clkdiv, hhg_pwm_get_default_config, hhg_pwm_gpio_to_slice_num, hhg_pwm_init, hhg_pwm_set_gpio_level};
use crate::drivers::gpio::{GpioConfig, GpioConfigs, GpioName, GpioInputType, InterruptCallback, InterruptConfig, InterruptType::{self, *}, GpioType};
use crate::traits::state::{Deinitializable, Initializable};
use GpioPippo::*;


const APP_TAG: &str = "PICO GPIO";
const GPIO_CONFIG_SIZE: usize = 7;

static GPIO_TABLE: [GpioConfig; GPIO_CONFIG_SIZE] = [
    GpioConfig::new(&EncoderA, GpioType::Input(None, 21, GpioInputType::PullDown), 0),
    GpioConfig::new(&EncoderB, GpioType::Input(None, 20, GpioInputType::PullDown), 0),
    GpioConfig::new(&EncoderBtn, GpioType::Input(None, 19, GpioInputType::PullUp), 0),
    GpioConfig::new(&Btn, GpioType::Input(None, 19, GpioInputType::PullDown), 0),
    GpioConfig::new(&LedRed, GpioType::OutputPWM(None, 13), 0),
    GpioConfig::new(&LedGreen, GpioType::OutputPWM(None, 14), 0),
    GpioConfig::new(&LedBlue, GpioType::OutputPWM(None, 15), 0),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioPippo {
    NoUsed,
    EncoderA,
    EncoderB,
    EncoderBtn,
    Btn,
    LedRed,
    LedGreen,
    LedBlue,
}
 
impl GpioName for GpioPippo {
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

impl FromStr for GpioPippo {
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

pub struct Gpio {
    gpio_configs: GpioConfigs<'static, GPIO_CONFIGS_SIZE>,
    idx: isize,
}
   
unsafe impl Send for Gpio {}
unsafe impl Sync for Gpio {}


impl Initializable for Gpio {
    fn init(&mut self) -> Result<()> {
        
        log_info!(APP_TAG, "Init GPIO");

        for i in 0..=self.idx {

            match self.gpio_configs[i] {

                Some(ref config) => {

                    match &config.get_io_type() {
                    
                        GpioType::Input(_, pin, input_type) => {
                            unsafe {
                                log_info!(APP_TAG, "Input: {}", config.get_name());

                                hhg_gpio_init(*pin);   
                                hhg_gpio_set_dir(*pin, GPIO_IN);

                                use GpioInputType::*;
                                match input_type {
                                    NoPull => (),
                                    PullUp => hhg_gpio_pull_up(*pin),
                                    PullDown => hhg_gpio_pull_down(*pin),
                                }

                                hhg_gpio_put(*pin, config.default_value != 0);
                            }
                        },
                    
                        GpioType::OutputPWM(_, pin) => {
                            
                            log_info!(APP_TAG, "Output PWM: {}", config.get_name());
                            
                            unsafe {
                                hhg_gpio_set_function(*pin, gpio_function_t::GPIO_FUNC_PWM as u32);
                                let slice_num = hhg_pwm_gpio_to_slice_num(*pin);
                                let mut config = hhg_pwm_get_default_config();
                                hhg_pwm_config_set_clkdiv(&mut config, 4.0);
                                hhg_pwm_init(slice_num, &mut config, false);
                            }
                        },
                    
                        GpioType::NotInitialized => return Err(Error::NullPtr),
                        GpioType::InputAnalog(_, _, _, _) => return Err(Error::NullPtr),
                        GpioType::Output(_, _) => return Err(Error::NullPtr),
                        GpioType::Pheriferal(_, _, _) => return Err(Error::NullPtr),
                    
                    }
                
                },

                None => return Err(Error::NotFound)
                
            }
        }

        Ok(())
    }
}

impl Deinitializable for Gpio {

    fn deinit(&mut self) -> Result<()> {
       
       Ok(())
    }
}




impl Gpio {
    pub fn new(gpio_configs: &GpioConfigs<'static, GPIO_CONFIGS_SIZE>) -> Self {
        Self {
            gpio_configs: gpio_configs.clone(),
            idx: GPIO_CONFIGS_SIZE as isize - 1,
        }
    }


    pub fn write(&self, name: &dyn GpioName, state: bool) -> OsalRsBool {
        unsafe {
            if let Some(config) = &self.gpio_configs[name] {
                match &config.get_io_type() {
                    GpioType::Output(_, pin) => {
                        hhg_gpio_put(*pin, state);
                        OsalRsBool::True
                    },
                    _ => OsalRsBool::False,
                }
            } else {
                OsalRsBool::False
            }
        }
    }

    pub fn read(&self, name: &dyn GpioName) -> Result<u32> {
        unsafe {
            if let Some(config) = &self.gpio_configs[name] {
                match &config.get_io_type() {
                    GpioType::Input(_, pin, _) => {
                        let value = hhg_gpio_get(*pin);
                        Ok(value as u32)
                    },
                    _ => Err(Error::InvalidType),
                }
            } else {
                Err(Error::NotFound)
            }
        }
    }

    pub fn set_pwm(&self, name: &dyn GpioName, pwm_duty_cycle: u16) -> OsalRsBool {
        unsafe {
            if let Some(config) = &self.gpio_configs[name] {
                match &config.get_io_type() {
                    GpioType::OutputPWM(_, pin) => {
                        hhg_pwm_set_gpio_level(*pin, pwm_duty_cycle);
                        OsalRsBool::True
                    },
                    _ => OsalRsBool::False,
                }
            } else {
                OsalRsBool::False
            }
        }
    }

    pub fn set_interrupt(
        &mut self, 
        name: &dyn GpioName,
        irq_type: InterruptType,
        enable: bool,
        callback: InterruptCallback
    ) -> OsalRsBool {

        use ffi::gpio_irq_level::*;

        if let Some(config) = &mut self.gpio_configs[name] {
            match &config.get_io_type() {
                GpioType::Input(_, pin, _) => {
                    

                    log_info!(APP_TAG, "Interrupt: {} enabled:{enable}", name.as_str());

                    unsafe {
                        match &irq_type {
                            RisingEdge => hhg_gpio_set_irq_enabled_with_callback(*pin, GPIO_IRQ_EDGE_RISE as u32, enable, callback),
                            FallingEdge => hhg_gpio_set_irq_enabled_with_callback(*pin, GPIO_IRQ_EDGE_FALL as u32, enable, callback),
                            BothEdge => hhg_gpio_set_irq_enabled_with_callback(*pin, GPIO_IRQ_EDGE_RISE as u32 | GPIO_IRQ_EDGE_FALL as u32, enable, callback),
                            HigthLevel => hhg_gpio_set_irq_enabled_with_callback(*pin, GPIO_IRQ_LEVEL_HIGH as u32, enable, callback),
                            LowLevel => hhg_gpio_set_irq_enabled_with_callback(*pin, GPIO_IRQ_LEVEL_LOW as u32, enable, callback),
                        }
                        hhd_irq_set_enabled(IO_IRQ_BANK0, true);
                    }

                    config.irq = Some(InterruptConfig::new(irq_type, enable, callback));
                    OsalRsBool::True
                },
                _ => OsalRsBool::False,
            }
        } else {
            OsalRsBool::False
        }
    
    }

    pub fn enable_interrupt(&mut self, name: &dyn GpioName, enable: bool) -> OsalRsBool {

        use ffi::gpio_irq_level::*;

        if let Some(config) = &mut self.gpio_configs[name] {
            match &config.get_io_type() {
                GpioType::Input(_, pin, _) => {
                    

                    match &mut config.irq {
                        Some(irq) => {

                            log_info!(APP_TAG, "Interrupt: {} enabled:{enable}", name.as_str());

                            unsafe {
                                match &irq.irq_type {
                                    RisingEdge => hhg_gpio_set_irq_enabled(*pin, GPIO_IRQ_EDGE_RISE as u32, enable),
                                    FallingEdge => hhg_gpio_set_irq_enabled(*pin, GPIO_IRQ_EDGE_FALL as u32, enable),
                                    BothEdge => hhg_gpio_set_irq_enabled(*pin, GPIO_IRQ_EDGE_RISE as u32 | GPIO_IRQ_EDGE_FALL as u32, enable),
                                    HigthLevel => hhg_gpio_set_irq_enabled(*pin, GPIO_IRQ_LEVEL_HIGH as u32, enable),
                                    LowLevel => hhg_gpio_set_irq_enabled(*pin, GPIO_IRQ_LEVEL_LOW as u32, enable),
                                }
                            }
                            irq.enable = enable;
                            OsalRsBool::True
                        }
                        None => OsalRsBool::False,
                    }

                    
                },
                _ => OsalRsBool::False,
            }
        } else {
            OsalRsBool::False
        }

    }

    pub fn len(&self) -> u32 {
        self.idx as u32 + 1
    }
}

