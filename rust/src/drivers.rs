#![allow(unused_imports)]
mod button;
mod encoder;
mod gpio; 

#[cfg(feature = "pico")]
mod pico;


#[cfg(feature = "pico")]
use crate::drivers::pico as plt;

pub mod platform {
    use osal_rs::os::ToPriority;
    use osal_rs::os::types::UBaseType;

    pub use crate::drivers::plt::gpio::*;
    pub use crate::drivers::plt::hardware::*;
    pub use crate::drivers::button::*;
    pub use crate::drivers::encoder::*;
    pub use crate::drivers::gpio::*;


    
    #[cfg(feature = "pico")]
    #[allow(dead_code)]
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum OaslThreadPriority {
        None = 0,
        Idle = 1,
        Low = 4,
        BelowNormal = 8,
        Normal = 12,
        AboveNormal = 16,
        BelowHigh = 19,
        High = 23,
        AboveHigh = 27,
        Realtime = 31,
    }
    
    impl ToPriority for OaslThreadPriority {
        fn to_priority(&self) -> UBaseType {
            *self as UBaseType
        }
    }

}