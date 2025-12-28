#![allow(dead_code)]

use core::any::Any; 
use core::ops::{Index, IndexMut};
use core::usize;

use alloc::{string::String, sync::Arc};
use alloc::string::ToString;

use osal_rs::utils::{Bytes, Error, OsalRsBool, Ptr, Result};

pub type InterruptCallback = extern "C" fn();
pub type PeripheralData = Arc<dyn Any + Send + Sync>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InterruptType
{
	RisingEdge,
	FallingEdge,
	BothEdge,
	HigthLevel,
	LowLevel,
}

#[derive(Clone, Debug)]
pub struct InterruptConfig {
    pub irq_type: InterruptType,
    pub enable: bool,
    pub callback: InterruptCallback,
}

impl InterruptConfig {
    pub fn new(
        irq_type: InterruptType,
        enable: bool,
        callback: InterruptCallback,
    ) -> Self {
        Self {
            irq_type,
            enable,
            callback,
        }
    }   
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InputType
{
	NoPull,
	PullUp,
	PullDown
}

#[derive(Clone, Debug)]
pub enum Type {
    NotInitialized,
    Input(Option<Ptr>, u32, InputType), //base, pin, pull
    InputAnalog(Option<Ptr>, u32, u32, u32), //base, pin, channel, ranck
    Output(Option<Ptr>, u32), 
    OutputPWM(Option<Ptr>, u32),
    Pheriferal(Option<Ptr>, u32, PeripheralData)
}




#[derive(Clone)]
pub struct GpioConfig<const NAME_SIZE: usize = 16> {
    name : Bytes<NAME_SIZE>,
    io_type: Type,
    pub default_value: u32,
    pub irq: Option<InterruptConfig>,
} 


impl<const NAME_SIZE: usize> PartialEq for GpioConfig<NAME_SIZE> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<const NAME_SIZE: usize> ToString for GpioConfig<NAME_SIZE> {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

impl<const NAME_SIZE: usize> GpioConfig<NAME_SIZE> {
    
    pub fn default() -> Self {
        use Type::*;
        Self { 
            name: Bytes::new_by_str(""), 
            io_type: NotInitialized, 
            default_value: 0, 
            irq: None,        
        }
    }

    pub fn new(
        name : &dyn ToString,
        io_type: Type,
        default_value: u32,
    ) -> Self {
        let name = name.to_string();

        Self {
            name: Bytes::new_by_string(&name),
            io_type,
            default_value,
            irq: None
        }
    }

    pub fn get_io_type(&self) -> Type {
        self.io_type.clone()
    }
}

#[derive(Clone)]
pub struct GpioConfigs<const SIZE: usize> {
    array: [Option<GpioConfig>; SIZE],
    index: usize,
}


impl<const SIZE: usize> Index<&dyn ToString> for GpioConfigs<SIZE> {
    type Output = Option<GpioConfig>;

    fn index(&self, name: &dyn ToString) -> &Self::Output {
        
        self.array.iter()
            .find(|it| {
                if let Some(config) = it {
                    config.name.to_string() == name.to_string()
                } else {
                    false
                }
            })
            .unwrap_or(&None)
    }
}

impl<const SIZE: usize> IndexMut<&dyn ToString> for GpioConfigs<SIZE> {

    fn index_mut(&mut self, name: &dyn ToString) -> &mut Self::Output {
        
        let mut index_find = -1isize;

        for (idx, it ) in &mut self.array.iter().enumerate() {
            if let Some(config) = it {
                if config.name.to_string() == name.to_string() {
                    index_find = idx as isize;  
                }
            }
        }
        
        if index_find > -1 {
            &mut self.array[index_find as usize]
        } else {
            let ret = &mut self.array[self.index];
            self.index += 1;
            ret
        }
    }
}


impl<const SIZE: usize> GpioConfigs<SIZE> {

    pub fn new() -> Self {
        Self{
            array: [const {None}; SIZE],
            index: 0,
        }
    }

    pub fn push(&mut self, config: GpioConfig) -> Result<String> {

        if self.index >= SIZE {
            return Err(Error::OutOfIndex)
        }

        for (i, it) in self.array.iter().enumerate() {
            if let Some(c) = it {
                if c.name == config.name {
                     self.array[i] = Some(config.clone());
                     return Ok(config.name.to_string())
                }
            }
        }

        self.array[self.index] = Some(config.clone());
        self.index += 1;
        

        Ok(config.name.to_string())
    }

}

pub trait Gpio : Sized + Sync + Send {
    fn new() -> Self
    where 
        Self: Sized;

    fn write(&self, name: &dyn ToString, state: bool) -> OsalRsBool;

    fn read(&self, name: &dyn ToString) -> Result<u32>;

    fn set_pwm(&self, name: &dyn ToString, pwm_duty_cycle: u16) -> OsalRsBool;

    fn set_interrupt(
        &mut self, 
        name: &dyn ToString,
        irq_type: InterruptType,
        enable: bool,
        callback: InterruptCallback
    ) -> OsalRsBool;

    fn enable_interrupt(&mut self, name: &dyn ToString, anable: bool) -> OsalRsBool;

    fn len(&self) -> u32;
}