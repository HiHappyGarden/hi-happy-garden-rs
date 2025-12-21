use core::ptr::null_mut;

use alloc::vec::{self, Vec};
use hhg_traits::gpio::{GpioConfig, GpioConfigs, GpioFn, Type};

const NAME_SIZE : usize = 16usize;


pub struct Gpio {
    gpio_configs: GpioConfigs<'static>
}


impl GpioFn for Gpio {

    fn new() -> Self {

        let mut gpio_configs = GpioConfigs::new();
        
        if let Ok(a) = GpioConfig::<NAME_SIZE>::new("ENCODER_A", Type::Input,0, null_mut(), 1, None, None, None) {
            gpio_configs.push(a);
        }

        

        Self { 
            gpio_configs 
        }
    }
}