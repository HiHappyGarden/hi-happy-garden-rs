

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
    pub enum gpio_function_t {
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

    unsafe extern "C" {
        pub(super) fn hhg_gpio_init(gpio: u32);
        pub(super) fn hhg_gpio_set_dir(gpio: u32, out: bool);
        pub(super) fn hhg_gpio_put(gpio: u32, value: bool);
        pub(super) fn hhg_gpio_pull_up(gpio: u32);
        pub(super) fn hhg_gpio_pull_down(gpio: u32);
        pub(super) fn hhg_gpio_disable_pulls(gpio: u32);
        pub(super) fn hhg_gpio_set_function(gpio: u32, fn_: u32);
        pub(super) fn hhg_pwm_gpio_to_slice_num(gpio: u32) -> u32;
        pub(super) fn hhg_pwm_get_default_config() -> pwm_config;
        pub(super) fn hhg_pwm_config_set_clkdiv(c: *mut pwm_config, div: f32);
        pub(super) fn hhg_pwm_init(slice_num: u32, c: *mut pwm_config, start: bool);
    }   
}


use core::str::FromStr;

use alloc::string::{String, ToString};
use osal_rs::log_info;
use osal_rs::utils::{Error, Result};
use ffi::hhg_gpio_init;
use crate::drivers::pico::gpio::ffi::{GPIO_IN, gpio_function_t, hhg_gpio_pull_down, hhg_gpio_pull_up, hhg_gpio_put, hhg_gpio_set_dir, hhg_gpio_set_function, hhg_pwm_config_set_clkdiv, hhg_pwm_get_default_config, hhg_pwm_gpio_to_slice_num, hhg_pwm_init};
use crate::traits::gpio::{Gpio as GpioFn, GpioConfig, GpioConfigs, Type, InterruptCallback, InputType};
use crate::traits::state::{Deinitializable, Initializable};
use GpioType::*;


const NAME_MAX_SIZE: usize = 16usize;
const GPIO_CONFIGS_SIZE: usize = 10usize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioType {
    NoUsed,
    EncoderA,
    EncoderB,
    EncoderBtn,
    Btn,
    LedRed,
    LedGreen,
    LedBlue,
}
 
impl ToString for GpioType {
    fn to_string(&self) -> String {
        match self {
            NoUsed => "NoUsed".to_string(),
            EncoderA => "EncoderA".to_string(),
            EncoderB => "EncoderB".to_string(),
            EncoderBtn => "EncoderBtn".to_string(),
            Btn => "Btn".to_string(),
            LedRed => "LedRed".to_string(),
            LedGreen => "LedGreen".to_string(),
            LedBlue => "LedBlue".to_string(),
        }
    }
}

impl FromStr for GpioType {
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
    names: [GpioType; GPIO_CONFIGS_SIZE],
    gpio_configs: GpioConfigs<GPIO_CONFIGS_SIZE>,
    idx: isize,
}
   


impl Initializable for Gpio {
    fn init(&mut self) -> Result<()> {
        

        for i in 0..=self.idx {
            let idx = self.names[i as usize];

            match self.gpio_configs[&idx] {

                Some(ref config) => {

                    log_info!("GPIO", "Initializing GPIO: {}", config.clone().to_string());

                    match &config.get_io_type() {
                    
                        Type::Input(_, pin, input_type) => {
                            unsafe {
                                hhg_gpio_init(*pin);   
                                hhg_gpio_set_dir(*pin, GPIO_IN);

                                match input_type {
                                    InputType::NoPull => (),
                                    InputType::PullUp => hhg_gpio_pull_up(*pin),
                                    InputType::PullDown => hhg_gpio_pull_down(*pin),
                                }

                                hhg_gpio_put(*pin, config.default_value != 0);
                            }
                        },
                    
                        Type::OutputPWM(_, pin) => {
                            unsafe {
                                hhg_gpio_set_function(*pin, gpio_function_t::GPIO_FUNC_PWM as u32);
                                let slice_num = hhg_pwm_gpio_to_slice_num(*pin);
                                let mut config = hhg_pwm_get_default_config();
                                hhg_pwm_config_set_clkdiv(&mut config, 4.0);
                                hhg_pwm_init(slice_num, &mut config, false);
                            }
                        },
                    
                        Type::NotInitialized => return Err(Error::NullPtr),
                        Type::InputAnalog(_, _, _, _) => return Err(Error::NullPtr),
                        Type::Output(_, _) => return Err(Error::NullPtr),
                        Type::Pheriferal(_, _, _) => return Err(Error::NullPtr),
                    
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

impl GpioFn for Gpio {
    fn new() -> Self {

        let mut names = [GpioType::NoUsed; GPIO_CONFIGS_SIZE];
        let mut gpio_configs = GpioConfigs::new();
        
        if let Ok(name) = gpio_configs.push(
            GpioConfig::<NAME_MAX_SIZE>::new(
            &EncoderA, 
            Type::Input(None, 21, InputType::PullDown), 
            0)
        ) {
            names[0] = GpioType::from_str(&name).unwrap();
        }

        if let Ok(name) = gpio_configs.push(
            GpioConfig::<NAME_MAX_SIZE>::new(
            &EncoderB, 
            Type::Input(None, 20, InputType::PullDown), 
            0)
        ) {
            names[1] = GpioType::from_str(&name).unwrap();
        }

        if let Ok(name) = gpio_configs.push(
            GpioConfig::<NAME_MAX_SIZE>::new(
            &EncoderBtn, 
            Type::Input(None, 19, InputType::PullUp), 
            0)
        ) {
            names[2] = GpioType::from_str(&name).unwrap();
        }

        if let Ok(name) = gpio_configs.push(
            GpioConfig::<NAME_MAX_SIZE>::new(
            &Btn, 
            Type::Input(None, 19, InputType::PullUp), 
            0)
        ) {
            names[3] = GpioType::from_str(&name).unwrap();
        }

        if let Ok(name) = gpio_configs.push(
            GpioConfig::<NAME_MAX_SIZE>::new(
            &LedRed, 
            Type::OutputPWM(None, 13), 
            0)
        ) {
            names[4] = GpioType::from_str(&name).unwrap();
        }

        if let Ok(name) = gpio_configs.push(
            GpioConfig::<NAME_MAX_SIZE>::new(
            &LedGreen, 
            Type::OutputPWM(None, 14), 
            0)
        ) {
            names[5] = GpioType::from_str(&name).unwrap();
        }

        if let Ok(name) = gpio_configs.push(
            GpioConfig::<NAME_MAX_SIZE>::new(
            &LedBlue, 
            Type::OutputPWM(None, 15), 
            0)
        ) {
            names[6] = GpioType::from_str(&name).unwrap();
        }


        Self {
            names,
            gpio_configs,
            idx: 6,
        }
    }


    fn write(&self, name: &dyn ToString, state: bool) -> osal_rs::utils::OsalRsBool {
        todo!()
    }

    fn read(&self, name: &dyn ToString, state: bool) -> Result<u32> {
        todo!()
    }

    fn set_interrupt(
        &mut self, 
        name: &dyn ToString,
        interrupt_type: crate::traits::gpio::InterruptType,
        interrupt_enable: bool,
        interrupt_callback: Option<InterruptCallback>
    ) -> osal_rs::utils::OsalRsBool {
        todo!()
    }

    fn enable_interrupt(&mut self, name: &dyn ToString, anable: bool) -> osal_rs::utils::OsalRsBool {
        todo!()
    }

    fn len(&self) -> u32 {
        self.idx as u32 + 1
    }
}

