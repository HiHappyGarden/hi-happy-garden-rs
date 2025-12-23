
use core::ops::{Index};
use core::ptr::null_mut;

use alloc::string::ToString;
use alloc::sync::Arc;

use osal_rs::from_str_to_array;
use osal_rs::utils::{Ptr};


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
        name : &dyn ToString,
        io_type: Type<'a>,
        default_value: u8,
        gpio_base: Ptr,
        pin: u32,
        adc_channel: Option<u32>,
        adc_rank: Option<u32>,
        interrupt_callback: Option<InterruptCallback>
    ) -> Self {
        
            // let mut name_array = [b' '; NAME_SIZE];
            // let bytes = name.as_bytes();
            // if name.len() > NAME_SIZE {
            //     name_array[..bytes.len()].copy_from_slice(bytes[..NAME_SIZE]);
            // } else {
            //     name_array[..bytes.len()].copy_from_slice(bytes);
            // }


            let name = name.to_string();

            from_str_to_array!(&name, name_array, NAME_SIZE);
            Self {
                name: name_array,
                io_type,
                default_value,
                gpio_base,
                pin,
                adc_channel,
                adc_rank,
                interrupt_callback
            }
        
    }
}

#[derive(Clone)]
pub struct GpioConfigs<'a, const SIZE: usize> {
    array: [Option<GpioConfig<'a>>; SIZE],
    index: usize,
    no_found: Option<GpioConfig<'a>>
}


impl<'a, const SIZE: usize> Index<&dyn ToString> for GpioConfigs<'a, SIZE> {
    type Output = Option<GpioConfig<'a>>;

    fn index(&self, name: &dyn ToString) -> &Self::Output {
        let name_bytes = name.to_string();
        let name_bytes = name_bytes.as_bytes();
        
        self.array.iter()
            .find(|it| {
                if let Some(config) = it {
                    config.name == name_bytes
                } else {
                    false
                }
            })
            .unwrap_or(&None)
    }
}

// impl<'a, const SIZE: usize> IndexMut<&dyn ToString> for GpioConfigs<'a, SIZE> {
    
//     fn index_mut(&mut self, name: &dyn ToString) -> &mut Self::Output {
//         let name_bytes = name.to_string();
//         let name_bytes = name_bytes.as_bytes();

//         for (i, it) in self.array.iter().enumerate() {
//             if let Some(config) = it {
//                 if config.name == name_bytes {
//                     return &mut self.array[i];
//                 }
//             }
//         }

//         &mut self.no_found
//     }
// }

impl<'a, const SIZE: usize> GpioConfigs<'a, SIZE> {

    pub fn new() -> Self {
        Self{
            array: [const {None}; SIZE],
            index: 0,
            no_found: const {None}
        }
    }

    pub fn push(&mut self, config: GpioConfig<'a>) -> bool {

        if self.index >= SIZE {
            return false
        }

        for (i, it) in self.array.iter().enumerate() {
            if let Some(c) = it {
                if c.name == config.name {
                     self.array[i] = Some(config.clone());
                     self.index += 1;
                     return true
                }
            }
        }

        false
    }

}

pub trait Gpio {
    fn new() -> Self
    where 
        Self: Sized;

    

}