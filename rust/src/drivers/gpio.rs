#![allow(dead_code)]

use core::any::Any; 
use core::fmt::write;
use core::ops::{Index, IndexMut};
use core::usize;

use alloc::str;
use alloc::sync::Arc;

use osal_rs::{log_info, log_warning};
use osal_rs::utils::{Error, OsalRsBool, Ptr, Result};

use crate::traits::state::{Deinitializable, Initializable};

const APP_TAG: &str = "GPIO";

//// Interrupt Configuration ////

pub type InterruptCallback = extern "C" fn();

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InterruptType
{
	RisingEdge,
	FallingEdge,
	BothEdge,
	HigthLevel,
	LowLevel,
}

//// GPIO ////

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GpioInputType
{
	NoPull,
	PullUp,
	PullDown
}

#[derive(Clone, Debug)]
pub struct InterruptConfig {
    pub irq_type: InterruptType,
    pub enable: bool,
    pub callback: InterruptCallback,
}

impl InterruptConfig {
    pub const fn new(
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

pub type GpioPeripheralData = Arc<dyn Any + Send + Sync>;

#[derive(Clone, Debug)]
pub enum GpioType {
    NotInitialized,
    Input(Option<Ptr>, u32, GpioInputType), //base, pin, pull
    InputAnalog(Option<Ptr>, u32, u32, u32), //base, pin, channel, ranck
    Output(Option<Ptr>, u32), //base, pin
    OutputPWM(Option<Ptr>, u32), //base, pin
    Pheriferal(Option<Ptr>, u32, GpioPeripheralData) //base, pin, peripheral data
}


pub trait GpioName : Sync + Send { 
    fn as_str(&self) -> &str;
}

enum GpioNameEmpty { Empty }

impl GpioName for GpioNameEmpty {
    fn as_str(&self) -> &str {
        ""
    }
}

#[derive(Clone)]
pub struct GpioInputTypeFn {
    pub no_pull: Option<fn(Option<Ptr>, u32)>,
    pub pull_up: Option<fn(Option<Ptr>, u32)>,
    pub pull_down: Option<fn(Option<Ptr>, u32)>,
}

unsafe impl Send for GpioInputTypeFn {}
unsafe impl Sync for GpioInputTypeFn {}

#[derive(Clone)]
pub struct GpioFn {
    pub init: Option<fn() -> Result<()>>,
    pub input: Option<fn(Option<Ptr>, u32, GpioInputType) -> Result<u32>>,
    pub input_analog: Option<fn(Option<Ptr>, u32, u32, u32) -> Result<u32>>,
    pub output: Option<fn(Option<Ptr>, u32, u32) -> Result<()>>,
    pub output_pwm: Option<fn(Option<Ptr>, u32, u32) -> Result<()>>,
    pub peripheral: Option<fn(Option<Ptr>, u32, GpioPeripheralData) -> Result<()>>,
    pub deinit: Option<fn() -> Result<()>>,
    pub read: Option<fn(Option<Ptr>, u32) -> Result<u32>>,
    pub write: Option<fn(Option<Ptr>, u32, u32) -> Result<()>>,
    pub set_pwm: Option<fn(Option<Ptr>, u32, f32) -> Result<()>>,
    pub set_interrupt: Option<fn(Option<Ptr>, u32, InterruptType, InterruptCallback, bool) -> Result<()>>,
    pub enable_interrupt: Option<fn(Option<Ptr>, u32, bool) -> Result<()>>,
}

unsafe impl Send for GpioFn {}
unsafe impl Sync for GpioFn {}


pub struct Gpio<'a, const GPIO_CONFIG_SIZE: usize> {
    functions: &'a GpioFn,
    input_type_functions: &'a GpioInputTypeFn,
    configs: &'a mut GpioConfigs<'a, GPIO_CONFIG_SIZE>,
    idx: isize,
}

unsafe impl<const GPIO_CONFIG_SIZE: usize> Send for Gpio<'_, GPIO_CONFIG_SIZE> {}
unsafe impl<const GPIO_CONFIG_SIZE: usize> Sync for Gpio<'_, GPIO_CONFIG_SIZE> {}


impl<const GPIO_CONFIG_SIZE: usize> Initializable for Gpio<'_, GPIO_CONFIG_SIZE> {
    fn init(&mut self) -> Result<()> {
        
        log_info!(APP_TAG, "Init GPIO");

        if let Some(init) = self.functions.init {
            init()?;
        }

        for i in 0..=self.idx {

            match self.configs[i] {

                Some(ref config) => {

                    match &config.get_io_type() {
                    
                        GpioType::Input(base, pin, input_type) => {
                            
                            

                            if let Some(input) = self.functions.input {
                                log_info!(APP_TAG, "Input: {}", config.get_name());

                                input(*base, *pin, *input_type)?;

                                use GpioInputType::*;
                                match input_type {
                                    NoPull => if let Some(no_pull) = self.input_type_functions.no_pull {
                                        no_pull(*base, *pin);
                                    },
                                    PullUp => if let Some(pull_up) = self.input_type_functions.pull_up {
                                        pull_up(*base, *pin);
                                    },
                                    PullDown => if let Some(pull_down) = self.input_type_functions.pull_down {
                                        pull_down(*base, *pin);
                                    },
                                }

                                if let Some(write) = self.functions.write {
                                    write(*base, *pin, config.default_value);
                                }
                            } else {
                                log_warning!(APP_TAG, "Input function not defined for: {}", config.get_name());
                            }


                        }
                    
                        GpioType::OutputPWM(base, pin) => {
                            
                            if let Some(output_pwm) = self.functions.output_pwm {
                                log_info!(APP_TAG, "Output PWM: {}", config.get_name());
                                output_pwm(*base, *pin, config.default_value)?;
                            } else {
                                log_warning!(APP_TAG, "Output PWM function not defined for: {}", config.get_name());
                            }
                        },
                    
                        GpioType::NotInitialized => {
                            
                            log_info!(APP_TAG, "Not Initialized: {}", config.get_name());
                        
                        },
                        GpioType::InputAnalog(base, pin, channel, ranck) => {
                            
                            if let Some(input_analog) = self.functions.input_analog {
                                log_info!(APP_TAG, "Input Analog: {}", config.get_name());
                                input_analog(*base, *pin, *channel, *ranck)?;
                            } else {
                                log_warning!(APP_TAG, "Input Analog function not defined for: {}", config.get_name());
                            }
                        },
                        GpioType::Output(base, pin) => {
                            
                            
                            
                            if let Some(output) = self.functions.output {
                                log_info!(APP_TAG, "Output: {}", config.get_name());
                                output(*base, *pin, config.default_value)?;
                            } else {
                                log_warning!(APP_TAG, "Output function not defined for: {}", config.get_name());
                            }
                        },
                        GpioType::Pheriferal(base, pin, peripheral_data) => {

                            if let Some(peripheral) = self.functions.peripheral {
                                log_info!(APP_TAG, "Peripheral: {}", config.get_name());
                                peripheral(*base, *pin, peripheral_data.clone())?;
                            } else {
                                log_warning!(APP_TAG, "Peripheral function not defined for: {}", config.get_name());
                            }
                        },
                    
                    }
                
                },

                None => return Err(Error::NotFound)
                
            }
        }

        Ok(())
    }
}

impl<const GPIO_CONFIG_SIZE: usize> Deinitializable for Gpio<'_, GPIO_CONFIG_SIZE> {

    fn deinit(&mut self) -> Result<()> {
       
        if let Some(deinit) = self.functions.deinit {
            deinit()?;
        } else {
            log_warning!(APP_TAG, "Deinit function not defined");
        } 
        Ok(())
    }
}



impl<'a, const GPIO_CONFIG_SIZE: usize> Gpio<'a, GPIO_CONFIG_SIZE> {
    pub const fn new(functions: &'a GpioFn, input_type_functions: &'a GpioInputTypeFn, configs: &'a mut GpioConfigs<'a, GPIO_CONFIG_SIZE>,) -> Self {
        Self {
            functions,
            input_type_functions,
            configs,
            idx: GPIO_CONFIG_SIZE as isize - 1,
        }
    }


    pub fn write(&self, name: &dyn GpioName, state: bool) -> OsalRsBool {

        if let Some(config) = &self.configs[name] {
            match &config.get_io_type() {
                GpioType::Output(base, pin) => 
                    
                    match &self.functions.write {
                        Some(write) => 
                            if write(*base, *pin, if state { 1 } else { 0 }).is_ok() {
                                OsalRsBool::True
                            } else {
                                OsalRsBool::False   
                            }
                        ,
                        None => OsalRsBool::False,
                    }

                ,
                _ => OsalRsBool::False,
            }
        } else {
            OsalRsBool::False
        }

    }

    pub fn read(&self, name: &dyn GpioName) -> Result<u32> {
        
        if let Some(config) = &self.configs[name] {
            match &config.get_io_type() {
                GpioType::Input(base, pin, _) => {
                    if let Some(read) = self.functions.read {
                        return read(*base, *pin).map_err(|_| Error::Unhandled("GPIO Read Error"));
                    } else {
                        return Err(Error::NotFound);
                    }
                }
                _ => Err(Error::InvalidType),
            }
        } else {
            Err(Error::NotFound)
        }
        
    }

    pub fn set_pwm(&self, name: &dyn GpioName, pwm_duty_cycle: u16) -> OsalRsBool {

        if let Some(config) = &self.configs[name] {
            match &config.get_io_type() {
                GpioType::OutputPWM(base, pin) => 
                    if let Some(set_pwm) = self.functions.set_pwm {
                        if set_pwm(*base, *pin, pwm_duty_cycle as f32).is_ok() {
                            OsalRsBool::True
                        } else {
                            OsalRsBool::False   
                        }
                    } else {
                        OsalRsBool::False
                    }
                ,
                _ => OsalRsBool::False,
            }
        } else {
            OsalRsBool::False
        }
        
    }

    pub fn set_interrupt(
        &mut self, 
        name: &dyn GpioName,
        irq_type: InterruptType,
        enable: bool,
        callback: InterruptCallback
    ) -> OsalRsBool {



        if let Some(config) = &mut self.configs[name] {
            match &config.get_io_type() {
                GpioType::Input(base, pin, input_type) => {
                    

                    log_info!(APP_TAG, "Interrupt: {} enabled:{enable}", name.as_str());

                    let ret : OsalRsBool;
                    if let Some(set_interrupt) = self.functions.set_interrupt {
                        
                        if set_interrupt(*base, *pin, irq_type.clone(), callback, enable).is_ok() {
                            ret = OsalRsBool::True;
                        } else {
                            ret = OsalRsBool::False;
                        }

                    } else {
                        return OsalRsBool::False
                    } 

                    if ret == OsalRsBool::False {
                        return ret;
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

        if let Some(config) = &mut self.configs[name] {
            match &config.get_io_type() {
                GpioType::Input(base, pin, _) => {
                    

                    match &mut config.irq {
                        Some(irq) => {

                            log_info!(APP_TAG, "Interrupt: {} enabled:{enable}", name.as_str());


                            let ret : OsalRsBool;
                            if let Some(enable_interrupt) = self.functions.enable_interrupt {
                                
                                if enable_interrupt(*base, *pin, enable).is_ok() {
                                    ret = OsalRsBool::True;
                                } else {
                                    ret = OsalRsBool::False;
                                }

                            } else {
                                return OsalRsBool::False
                            } 

                            if ret == OsalRsBool::False {
                                return ret;
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


//// GPIO Configuration ////

#[derive(Clone)]
pub struct GpioConfig<'a> {
    name : &'a dyn GpioName,
    io_type: GpioType,
    pub default_value: u32,
    pub irq: Option<InterruptConfig>,
} 

unsafe impl Sync for GpioConfig<'_> {}
unsafe impl Send for GpioConfig<'_> {}


impl PartialEq for GpioConfig<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name.as_str() == other.name.as_str()
    }
}

impl<'a> GpioConfig<'a> {
    
    pub fn default() -> Self {
        Self { 
            name: &GpioNameEmpty::Empty, 
            io_type: GpioType::NotInitialized, 
            default_value: 0, 
            irq: None,        
        }
    }

    pub const fn new (
        name : &'a dyn GpioName,
        io_type: GpioType,
        default_value: u32,
    ) -> Self {

        Self {
            name: name,
            io_type,
            default_value,
            irq: None
        }
    }

    pub fn get_io_type(&self) -> GpioType {
        self.io_type.clone()
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Clone)]
pub struct GpioConfigs<'a, const SIZE: usize> {
    array: [Option<GpioConfig<'a>>; SIZE],
    index: usize,
}


impl<'a, const SIZE: usize> Index<&dyn GpioName> for GpioConfigs<'a, SIZE> {
    type Output = Option<GpioConfig<'a>>;

    fn index(&self, name: &dyn GpioName) -> &Self::Output {
        
        self.array.iter()
            .find(|it| {
                if let Some(config) = it {
                    config.name.as_str() == name.as_str()
                } else {
                    false
                }
            })
            .unwrap_or(&None)
    }
}

impl<'a, const SIZE: usize> Index<isize> for GpioConfigs<'a, SIZE> {
    type Output = Option<GpioConfig<'a>>;

    fn index(&self, name: isize) -> &Self::Output {
        &self.array[name as usize]
    }
}

impl<'a, const SIZE: usize> IndexMut<&dyn GpioName> for GpioConfigs<'a, SIZE> {

    fn index_mut(&mut self, name: &dyn GpioName) -> &mut Self::Output {
        
        let mut index_find = -1isize;

        for (idx, it ) in &mut self.array.iter().enumerate() {
            if let Some(config) = it {
                if config.name.as_str() == name.as_str() {
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


impl<'a, const SIZE: usize> GpioConfigs<'a, SIZE> {

    pub const fn new() -> Self {
        Self{
            array: [const {None}; SIZE],
            index: 0,
        }
    }

    pub fn push(&mut self, config: GpioConfig<'a>) -> Result<&'a str> {

        if self.index >= SIZE {
            return Err(Error::OutOfIndex)
        }

        for (i, it) in self.array.iter().enumerate() {
            if let Some(c) = it {
                if c.name.as_str() == config.name.as_str() {
                     self.array[i] = Some(config.clone());
                     return Ok(config.name.as_str())
                }
            }
        }

        self.array[self.index] = Some(config.clone());
        self.index += 1;
        

        Ok(config.name.as_str())
    }

}

