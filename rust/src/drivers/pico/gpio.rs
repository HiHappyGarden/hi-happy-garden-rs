use core::ptr::null_mut;

use alloc::string::{String, ToString};
use osal_rs::utils::Result;
use crate::traits::{gpio::{Gpio as GpioFn, GpioConfig, GpioConfigs, Type}, initializable::Initializable};

const NAME_SIZE : usize = 16usize;

pub enum GpioType {
    EncoderA,
    EncoderB,
    EncoderBtn,
    Btn,
    LedRed,
    LedGreen,
    LedBlue
}
 
impl ToString for GpioType {
    fn to_string(&self) -> String {
        use GpioType::*;
        match self {
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


pub struct Gpio {
    gpio_configs: GpioConfigs<'static, 10>
}


impl GpioFn for Gpio {

    fn new() -> Self {

        let mut gpio_configs = GpioConfigs::new();
        
        
        gpio_configs.push(GpioConfig::<NAME_SIZE>::new(&GpioType::EncoderA, Type::Input,0, null_mut(), 1, None, None, None));
        

        gpio_configs[&GpioType::EncoderA].clone();
        

        Self { 
            gpio_configs 
        }
    }
}

impl Initializable for Gpio {
    fn init(&mut self) -> Result<()> {
        

        Ok(())
    }
}