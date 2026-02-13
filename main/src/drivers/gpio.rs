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
 
 #![allow(dead_code)]

use core::any::Any; 
use core::ops::{Index, IndexMut};

use alloc::str;
use alloc::sync::Arc;

use osal_rs::os::{RawMutex, AsSyncStr};
use osal_rs::{log_info, log_warning};
use osal_rs::utils::{Error, OsalRsBool, Ptr, Result};


use crate::traits::state::{Deinitializable, Initializable};
use crate::drivers::platform::{GPIO_CONFIG_SIZE, GPIO_CONFIGS, GPIO_FN};

const APP_TAG: &str = "GPIO";

//// Interrupt Configuration ////

pub type InterruptCallback = extern "C" fn();

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InterruptType
{
	RisingEdge,
	FallingEdge,
	BothEdge,
	HighLevel,
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
    Input(Option<Ptr>, u32, GpioInputType, u32), //base, pin, gpioInputType, default value
    InputAnalog(Option<Ptr>, u32, u32, u32), //base, pin, channel, rank
    Output(Option<Ptr>, u32, u32), //base, pin, default value
    OutputPWM(Option<Ptr>, u32, u32), //base, pin, default value
    Peripheral(Option<Ptr>, u32, GpioPeripheralData) //base, pin, peripheral data
}



enum GpioNameEmpty { Empty }

impl AsSyncStr for GpioNameEmpty {
    fn as_str(&self) -> &str {
        ""
    }
}


#[derive(Clone)]
pub struct GpioFn {
    pub init: Option<fn() -> Result<()>>,
    pub input: Option<fn(&GpioConfig, Option<Ptr>, u32, GpioInputType, u32) -> Result<()>>,
    pub input_analog: Option<fn(&GpioConfig, Option<Ptr>, u32, u32, u32) -> Result<()>>,
    pub output: Option<fn(&GpioConfig, Option<Ptr>, u32, u32) -> Result<()>>,
    pub output_pwm: Option<fn(&GpioConfig, Option<Ptr>, u32, u32) -> Result<()>>,
    pub peripheral: Option<fn(&GpioConfig, Option<Ptr>, u32, GpioPeripheralData) -> Result<()>>,
    pub deinit: Option<fn() -> Result<()>>,
    pub read: Option<fn(&GpioConfig, Option<Ptr>, u32) -> Result<u32>>,
    pub write: Option<fn(&GpioConfig, Option<Ptr>, u32, u32) -> OsalRsBool>,
    pub set_pwm: Option<fn(&GpioConfig, Option<Ptr>, u32, u32) -> OsalRsBool>,
    pub set_interrupt: Option<fn(&GpioConfig, Option<Ptr>, u32, InterruptType, InterruptCallback, bool) -> OsalRsBool>,
    pub enable_interrupt: Option<fn(&GpioConfig, Option<Ptr>, u32, bool) -> OsalRsBool>,
}

unsafe impl Send for GpioFn {}
unsafe impl Sync for GpioFn {}


pub struct Gpio<const GPIO_CONFIG_SIZE: usize> {
    configs: &'static mut GpioConfigs<'static, GPIO_CONFIG_SIZE>,
    mutex: RawMutex,
}

unsafe impl<const GPIO_CONFIG_SIZE: usize> Send for Gpio<GPIO_CONFIG_SIZE> {}
unsafe impl<const GPIO_CONFIG_SIZE: usize> Sync for Gpio<GPIO_CONFIG_SIZE> {}


impl<const GPIO_CONFIG_SIZE: usize> Initializable for Gpio<GPIO_CONFIG_SIZE> {
    fn init(&mut self) -> Result<()> {
        
        log_info!(APP_TAG, "Init GPIO");

        if let Some(init) = &GPIO_FN.init {
            init()?;
        }

        for i in 0..=self.configs.idx() {

            match self.configs[i] {

                Some(ref config) => {

                    match &config.get_io_type() {
                        GpioType::NotInitialized => log_info!(APP_TAG, "Not Initialized:{}", config.get_name()),
                        GpioType::Input(base, pin, input_type, default_value) => 
                            
                            if let Some(input) = &GPIO_FN.input {
                                log_info!(APP_TAG, "Input:{}", config.get_name());

                                input(&config, *base, *pin, *input_type, *default_value)?;

                            } else {
                                log_warning!(APP_TAG, "Input function not defined for: {}", config.get_name());
                            }
                        ,

                        GpioType::InputAnalog(base, pin, channel, ranck) => 
                            
                            if let Some(input_analog) = &GPIO_FN.input_analog {
                                log_info!(APP_TAG, "Input Analog:{}", config.get_name());
                                input_analog(&config, *base, *pin, *channel, *ranck)?;
                            } else {
                                log_warning!(APP_TAG, "Input Analog function not defined for: {}", config.get_name());
                            }

                        ,
                        
                        GpioType::Output(base, pin, default_value) => 
    
                            if let Some(output) = &GPIO_FN.output {
                                log_info!(APP_TAG, "Output:{}", config.get_name());
                                output(&config, *base, *pin, *default_value)?;
                            } else {
                                log_warning!(APP_TAG, "Output function not defined for: {}", config.get_name());
                            }
                        ,
                    
                        GpioType::OutputPWM(base, pin, default_value) => 
                            
                            if let Some(output_pwm) = &GPIO_FN.output_pwm {
                                log_info!(APP_TAG, "Output PWM:{}", config.get_name());
                                output_pwm(&config, *base, *pin, *default_value)?;
                            } else {
                                log_warning!(APP_TAG, "Output PWM function not defined for: {}", config.get_name());
                            }
                        ,

                        GpioType::Peripheral(base, pin, peripheral_data) =>

                            if let Some(peripheral) = &GPIO_FN.peripheral {
                                log_info!(APP_TAG, "Peripheral:{}", config.get_name());
                                peripheral(&config, *base, *pin, peripheral_data.clone())?;
                            } else {
                                log_warning!(APP_TAG, "Peripheral function not defined for: {}", config.get_name());
                            }
                        
                    
                    }
                
                },
                None => return Err(Error::NotFound)
                
            }
        }

        Ok(())
    }
}

impl<const GPIO_CONFIG_SIZE: usize> Deinitializable for Gpio<GPIO_CONFIG_SIZE> {

    fn deinit(&mut self) -> Result<()> {
       
        if let Some(deinit) = &GPIO_FN.deinit {
            deinit()?;
        } else {
            log_warning!(APP_TAG, "Deinit function not defined");
        } 
        Ok(())
    }
}



impl Gpio<{GPIO_CONFIG_SIZE}> {
    pub fn new() -> Self {

        let mutex = match RawMutex::new() {
            Ok(mutex) => mutex,
            Err(_) => panic!("Failed to create GPIO mutex"),
        };

        Self {
            configs:  unsafe { &mut *(&raw mut GPIO_CONFIGS ) },
            mutex
        }
    }

    pub fn get_mutex(&self) -> &RawMutex {
        &self.mutex
    }
}

impl<const GPIO_CONFIG_SIZE: usize> Gpio<GPIO_CONFIG_SIZE> {


    pub fn write(&self, name: &dyn AsSyncStr, state: u32) -> OsalRsBool {

        if let Some(config) = &self.configs[name] {
            match &config.get_io_type() {
                GpioType::Output(base, pin, _) => 
                    
                    match &&GPIO_FN.write {
                        Some(write) => write(&config, *base, *pin, state),
                        None => OsalRsBool::False,
                    }

                ,
                _ => OsalRsBool::False,
            }
        } else {
            OsalRsBool::False
        }

    }

    pub fn read(&self, name: &dyn AsSyncStr) -> Result<u32> {
        
        if let Some(config) = &self.configs[name] {
            match &config.get_io_type() {
                GpioType::Input(base, pin, _, _) => {
                    if let Some(read) = &GPIO_FN.read {
                        read(&config, *base, *pin).map_err(|_| Error::Unhandled("GPIO Read Error"))
                    } else {
                        Err(Error::NotFound)
                    }
                }
                GpioType::InputAnalog(base, _, channel, _) => {
                    if let Some(read) = &GPIO_FN.read {
                        read(&config, *base, *channel).map_err(|_| Error::Unhandled("GPIO Read Error"))
                    } else {
                        Err(Error::NotFound)
                    }
                }
                _ => Err(Error::InvalidType),
            }
        } else {
            Err(Error::NotFound)
        }
        
    }

    pub fn set_pwm(&self, name: &dyn AsSyncStr, pwm_duty_cycle: u16) -> OsalRsBool {

        if let Some(config) = &self.configs[name] {
            match &config.get_io_type() {
                GpioType::OutputPWM(base, pin,_) => 
                    if let Some(set_pwm) = &GPIO_FN.set_pwm {
                        set_pwm(&config, *base, *pin, pwm_duty_cycle as u32)
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
        name: &dyn AsSyncStr,
        irq_type: InterruptType,
        enable: bool,
        callback: InterruptCallback
    ) -> OsalRsBool {



        if let Some(config) = &mut self.configs[name] {
            match &config.get_io_type() {
                GpioType::Input(base, pin, _, _) => {
                    

                    

                    let ret : OsalRsBool;
                    if let Some(set_interrupt) = &GPIO_FN.set_interrupt {
                        log_info!(APP_TAG, "Interrupt:{} type:{:?} enabled:{enable}", name.as_str(), irq_type);
                        ret = set_interrupt(&config, *base, *pin, irq_type.clone(), callback, enable);
                    
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

    pub fn enable_interrupt(&mut self, name: &dyn AsSyncStr, enable: bool) -> OsalRsBool {

        if let Some(config) = &mut self.configs[name] {
            match &config.get_io_type() {
                GpioType::Input(base, pin, _, _) => {
                    
                    let config_clone = config.clone();
                    match &mut config.irq {
                        Some(irq) => {

                            log_info!(APP_TAG, "Interrupt: {} enabled:{enable}", name.as_str());


                            let ret : OsalRsBool;
                            if let Some(enable_interrupt) = &GPIO_FN.enable_interrupt {
                                ret = enable_interrupt(&config_clone, *base, *pin, enable);
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
}


// //// GPIO Configuration ////

#[derive(Clone)]
pub struct GpioConfig<'a> {
    name : &'a dyn AsSyncStr,
    io_type: GpioType,
    pub irq: Option<InterruptConfig>,
} 

unsafe impl Sync for GpioConfig<'_> {}
unsafe impl Send for GpioConfig<'_> {}


impl PartialEq for GpioConfig<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name.as_str() == other.name.as_str()
    }
}

impl Default for GpioConfig<'_> {
    fn default() -> Self {
        Self { 
            name: &GpioNameEmpty::Empty, 
            io_type: GpioType::NotInitialized, 
            irq: None,        
        }
    }
}

impl<'a> GpioConfig<'a> {
    
    pub const fn new (
        name : &'a dyn AsSyncStr,
        io_type: GpioType,
    ) -> Self {

        Self {
            name: name,
            io_type,
            irq: None
        }
    }

    #[inline]
    pub fn get_io_type(&self) -> GpioType {
        self.io_type.clone()
    }

    #[inline]
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

//// GPIO Configuration Constainer ////

#[derive(Clone)]
pub struct GpioConfigs<'a, const GPIO_CONFIG_SIZE: usize> {
    array: [Option<GpioConfig<'a>>; GPIO_CONFIG_SIZE],
    index: isize,
}

impl<'a, const GPIO_CONFIG_SIZE: usize> Index<&dyn AsSyncStr> for GpioConfigs<'a, GPIO_CONFIG_SIZE> {
    type Output = Option<GpioConfig<'a>>;

    fn index(&self, name: &dyn AsSyncStr) -> &Self::Output {
        
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

impl<'a, const GPIO_CONFIG_SIZE: usize> Index<usize> for GpioConfigs<'a, GPIO_CONFIG_SIZE> {
    type Output = Option<GpioConfig<'a>>;

    fn index(&self, idx: usize) -> &Self::Output {
        if idx > self.index as usize {
            return &None;
        }
        
        &self.array[idx]
    }
}

impl<'a, const GPIO_CONFIG_SIZE: usize> IndexMut<&dyn AsSyncStr> for GpioConfigs<'a, GPIO_CONFIG_SIZE> {

    fn index_mut(&mut self, name: &dyn AsSyncStr) -> &mut Self::Output {
        
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
            let ret = &mut self.array[self.index as usize];
            self.index += 1;
            ret
        }
    }
}


impl<'a, const GPIO_CONFIG_SIZE: usize> GpioConfigs<'a, GPIO_CONFIG_SIZE> {

    pub const fn new() -> Self {
        Self{
            array: [const { None }; GPIO_CONFIG_SIZE],
            index: 0,
        }
    }

    pub const fn new_with_array(array: [Option<GpioConfig<'a>>; GPIO_CONFIG_SIZE]) -> Self {
        let index = array.len();
        let index = if index > GPIO_CONFIG_SIZE { GPIO_CONFIG_SIZE } else { index };
        let index = index as isize - 1;

        Self{
            array,
            index,
        }
    }

    pub fn push(&mut self, config: GpioConfig<'a>) -> Result<&'a str> {

        if self.index >= GPIO_CONFIG_SIZE as isize {
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

        self.array[self.index as usize] = Some(config.clone());
        self.index += 1;
        

        Ok(config.name.as_str())
    }


    #[inline]
    pub fn idx(&self) -> usize {
        if self.index < 0 {
            0usize
        } else {    
            self.index as usize
        }
    }
}

