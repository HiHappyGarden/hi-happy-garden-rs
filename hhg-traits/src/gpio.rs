use core::ptr::null_mut;

use alloc::sync::Arc;
use alloc::vec::Vec;
use osal_rs::utils::{Error, Result, Ptr};


#[derive(Clone, PartialEq, Eq)]
pub enum Type<'a> {
    NotInitialized,
    Input,
    InputAnalog,
    Output,
    OutputPWM,
    Pheriferal(&'a str)
}

type InterruptCallback = Arc<dyn Fn() + Send + Sync>;

#[allow(unused)]
#[derive(Clone)]
pub struct GpioConfig<'a, const NAME_SIZE: usize = 16> {
    name : [u8; NAME_SIZE],
    io_type: Type<'a>,
    default_value: u8,
    gpio_base: Ptr,
    pin: u32,
    adc_channel: Option<u32>,
    adc_rank: Option<u32>,
    interrupt_callback: Option<InterruptCallback>
} 

impl<'a, const NAME_SIZE: usize> PartialEq for GpioConfig<'a, NAME_SIZE> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[allow(unused)]
impl<'a, const NAME_SIZE: usize> GpioConfig<'a, NAME_SIZE> {
    pub const fn default() -> Self {
        Self { 
            name: [b' '; NAME_SIZE], 
            io_type: Type::NotInitialized, 
            default_value: 0, 
            gpio_base: null_mut(),
            pin: 0, 
            adc_channel: None, 
            adc_rank: None, 
            interrupt_callback: None 
        }
    }

    pub fn new(
        name : &str,
        io_type: Type<'a>,
        default_value: u8,
        gpio_base: Ptr,
        pin: u32,
        adc_channel: Option<u32>,
        adc_rank: Option<u32>,
        interrupt_callback: Option<InterruptCallback>
    ) -> Result<Self> {
        if name.len() > NAME_SIZE {
            Err(Error::Unhandled("name too big"))
        } else {
            let mut name_array = [b' '; NAME_SIZE];
            let bytes = name.as_bytes();
            name_array[..bytes.len()].copy_from_slice(bytes);
            
            Ok(Self {
                name: name_array,
                io_type,
                default_value,
                gpio_base,
                pin,
                adc_channel,
                adc_rank,
                interrupt_callback
            })
        }
    }
}

#[derive(Clone)]
pub struct GpioConfigs<'a> (Vec<GpioConfig<'a>>);

impl<'a> GpioConfigs<'a> {

    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get(&self, name: &str) -> Result<&GpioConfig<'a>> {
        match self.0.iter().find(|it| it.name == name.as_bytes() ) {
            Some(gpio_config) => Ok(gpio_config),
            None => Err(Error::NotFound),
        }
    }

    pub fn push(&mut self, value: GpioConfig<'a>) {
        self.0.push(value);
    }
}

pub trait GpioFn {
    fn new() -> Self
    where 
        Self: Sized;

    

}