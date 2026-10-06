mod button;
pub mod date_time;
mod encoder;
pub mod encrypt;
pub mod error;
pub mod filesystem;
pub mod gpio;
mod i2c;
mod lcd_sh1106;
pub mod network;
mod relays;
pub mod rgb_led;
mod rtc;
mod uart;
mod timer;
pub(super) mod wifi;

// The emulator runs the Pico 2 W platform layer unchanged: only the `hhg_*`
// functions under it (the C wrappers of src/pico on the board) are swapped
// for the host models of `emulator`
#[cfg(any(feature = "pico", feature = "emulator"))]
mod pico;

#[cfg(feature = "emulator")]
pub(crate) mod emulator;

#[cfg(any(feature = "pico", feature = "emulator"))]
use crate::drivers::pico as plt;

pub mod platform {
    pub(crate) use crate::drivers::plt::flash::*;
    pub(crate) use crate::drivers::plt::gpio::*;
    pub(crate) use crate::drivers::plt::hardware::*;
    pub(crate) use crate::drivers::plt::i2c::*;
    pub(in crate::drivers) use crate::drivers::plt::uart::*;

    pub type LCDDisplay = crate::drivers::lcd_sh1106::LCDSH1106;
    
    #[allow(unused)]
    pub const RTC_MINIMUM_DATE: i64 = crate::drivers::rtc::RTC::MINIMUM_DATE;    
}


/// Runs the on-target driver tests, see `crate::tests`.
///
/// Board health first (a failing peripheral explains the failures that
/// follow), then pure logic, then the tests that drive real peripherals.
#[cfg(feature = "tests")]
pub(crate) fn run_all_tests(stats: &mut crate::tests::TestStats) {
    error::tests::run_all_tests(stats);
    date_time::tests::run_all_tests(stats);
    encrypt::tests::run_all_tests(stats);
    filesystem::tests::run_all_tests(stats);
    i2c::tests::run_all_tests(stats);
    rtc::tests::run_all_tests(stats);
    gpio::tests::run_all_tests(stats);
    timer::tests::run_all_tests(stats);
    relays::tests::run_all_tests(stats);
}
