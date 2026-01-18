#![allow(unused_imports)]
mod button;
mod encoder;
pub mod filesystem;
pub mod gpio; 
mod i2c;
mod lcd_sh1106;
mod relays;
pub mod rgb_led;
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

    pub type LCDDisplay = crate::drivers::lcd_sh1106::LCDSH1106;
}

