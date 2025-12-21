use alloc::vec::{self, Vec};
use hhg_traits::gpio::{GpioConfig, GpioConfigs, GpioFn};




pub struct Gpio {
    gpio_configs: GpioConfigs<'static>
}

impl GpioFn for Gpio {
    fn new(gpio_configs: GpioConfigs) -> Self {
        let mut gpio_configs = GpioConfigs::new();

        // gpio_configs.push();

        Self { 
            gpio_configs 
        }
    }
}