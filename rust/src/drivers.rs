#![allow(unused_imports)]
mod button;
mod encoder;
mod gpio; 
mod i2c;
mod relays;
mod rgb_led;
mod uart;

#[cfg(feature = "pico")]
mod pico;


#[cfg(feature = "pico")]
use crate::drivers::pico as plt;

pub mod platform {
    pub use crate::drivers::plt::gpio::*;
    pub use crate::drivers::plt::hardware::*;
    pub use crate::drivers::plt::i2c::*;
    pub use crate::drivers::plt::uart::*;
    pub use crate::drivers::plt::lcd_sh1106::*;
    
    #[cfg(feature = "pico")]
    pub type LCDDisplay = LCDSH1106;
}

pub use crate::drivers::button::*;
pub use crate::drivers::encoder::*;
pub use crate::drivers::gpio::*;
pub use crate::drivers::i2c::*;
pub use crate::drivers::relays::*;
pub use crate::drivers::rgb_led::*;
pub use crate::drivers::uart::*;