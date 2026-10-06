/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! GPIO, PWM and ADC of the RP2350, plus the CYW43 wireless GPIOs.
//!
//! Twin of `src/pico/hhg-gpio-wrapper.c`. Nothing is wired to the pins, so an
//! input reads its pull unless a test or a developer drives it with
//! [`drive_input`], which also raises the configured interrupts.

use core::ffi::c_uint;
use core::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use crate::drivers::pico::ffi::gpio_function_type::{GPIO_FUNC_NULL, GPIO_FUNC_SIO};
use crate::drivers::pico::ffi::pwm_config;

use super::lock;

/// RP2350B pin count
const GPIO_COUNT: usize = 48;

// pico-sdk `gpio_irq_level` bits
const GPIO_IRQ_LEVEL_LOW: u32 = 0x1;
const GPIO_IRQ_LEVEL_HIGH: u32 = 0x2;
const GPIO_IRQ_EDGE_FALL: u32 = 0x4;
const GPIO_IRQ_EDGE_RISE: u32 = 0x8;

/// ADC channel of the internal temperature sensor
const ADC_TEMP_CHANNEL: u32 = 4;

/// 0.706 V on the 3.3 V / 12 bit ADC: 27 C by the RP2350 datasheet formula
const ADC_TEMP_27C: u16 = 876;

/// A floating ADC pin reads mid-scale
const ADC_FLOATING: u16 = 2_048;

/// CYW43 wireless GPIOs: WL_GPIO0 (LED), WL_GPIO1, WL_GPIO2
const WL_GPIO_COUNT: usize = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pull {
    None,
    Up,
    Down,
}

#[derive(Clone, Copy)]
struct Pin {
    function: u32,
    output: bool,
    out_level: bool,
    pull: Pull,
    /// Level forced from outside the board, `None` when nothing drives the pin
    driven: Option<bool>,
    pwm_level: u16,
    irq_events: u32,
    callback: Option<extern "C" fn()>,
}

impl Pin {
    /// RP2350 reset state: input, pull-down enabled, no function selected
    const RESET: Pin = Pin {
        function: GPIO_FUNC_NULL,
        output: false,
        out_level: false,
        pull: Pull::Down,
        driven: None,
        pwm_level: 0,
        irq_events: 0,
        callback: None,
    };

    fn level(&self) -> bool {
        if self.output {
            return self.out_level;
        }
        self.driven.unwrap_or(self.pull == Pull::Up)
    }
}

static PINS: Mutex<[Pin; GPIO_COUNT]> = Mutex::new([Pin::RESET; GPIO_COUNT]);

static WL_GPIOS: Mutex<[bool; WL_GPIO_COUNT]> = Mutex::new([false; WL_GPIO_COUNT]);

static ADC_INPUT: AtomicU32 = AtomicU32::new(0);

fn with_pin<R>(gpio: u32, f: impl FnOnce(&mut Pin) -> R) -> Option<R> {
    let mut pins = lock(&PINS);
    pins.get_mut(gpio as usize).map(f)
}

/// Drives an input pin from outside the board, as a pressed button or a
/// turning encoder would; `None` releases it back to its pull.
///
/// Edge and level interrupts enabled on the pin fire on the calling thread,
/// which plays the role of the IO_IRQ_BANK0 handler.
#[allow(dead_code)]
pub(crate) fn drive_input(gpio: u32, level: Option<bool>) {
    let fire = with_pin(gpio, |pin| {
        let before = pin.level();
        pin.driven = level;
        let after = pin.level();

        let events = pin.irq_events;
        let triggered = (!before && after && events & GPIO_IRQ_EDGE_RISE != 0)
            || (before && !after && events & GPIO_IRQ_EDGE_FALL != 0)
            || (after && events & GPIO_IRQ_LEVEL_HIGH != 0)
            || (!after && events & GPIO_IRQ_LEVEL_LOW != 0);

        if triggered { pin.callback } else { None }
    });

    // Called with the lock released: the handler reads the pins back
    if let Some(Some(callback)) = fire {
        callback();
    }
}

pub(in crate::drivers) unsafe fn hhg_gpio_init(gpio: u32) {
    with_pin(gpio, |pin| {
        pin.function = GPIO_FUNC_SIO;
        pin.output = false;
        pin.out_level = false;
    });
}

pub(in crate::drivers) unsafe fn hhg_gpio_set_dir(gpio: u32, out: bool) {
    with_pin(gpio, |pin| pin.output = out);
}

pub(in crate::drivers) unsafe fn hhg_gpio_put(gpio: u32, value: bool) {
    // As on the RP2350 the output latch is written on inputs too, it just
    // does not reach the pad until the pin turns into an output
    with_pin(gpio, |pin| pin.out_level = value);
}

pub(in crate::drivers) unsafe fn hhg_gpio_get(gpio: u32) -> bool {
    with_pin(gpio, |pin| pin.level()).unwrap_or(false)
}

pub(in crate::drivers) unsafe fn hhg_gpio_pull_up(gpio: u32) {
    with_pin(gpio, |pin| pin.pull = Pull::Up);
}

pub(in crate::drivers) unsafe fn hhg_gpio_pull_down(gpio: u32) {
    with_pin(gpio, |pin| pin.pull = Pull::Down);
}

pub(in crate::drivers) unsafe fn hhg_gpio_disable_pulls(gpio: u32) {
    with_pin(gpio, |pin| pin.pull = Pull::None);
}

pub(in crate::drivers) unsafe fn hhg_gpio_set_function(gpio: u32, fn_: u32) {
    with_pin(gpio, |pin| pin.function = fn_);
}

pub(in crate::drivers) unsafe fn hhg_pwm_gpio_to_slice_num(gpio: u32) -> u32 {
    // RP2350: GPIO 0..31 on slices 0..7, GPIO 32..47 on slices 8..11
    if gpio < 32 {
        (gpio >> 1) & 7
    } else {
        8 + ((gpio >> 1) & 3)
    }
}

pub(in crate::drivers) unsafe fn hhg_pwm_get_default_config() -> pwm_config {
    // pico-sdk defaults: free running, divider 1.0 (8.4 fixed point), wrap 0xffff
    pwm_config {
        csr: 0,
        div: 1 << 4,
        top: 0xffff,
    }
}

pub(in crate::drivers) unsafe fn hhg_pwm_config_set_clkdiv(c: *mut pwm_config, div: f32) {
    if let Some(c) = unsafe { c.as_mut() } {
        c.div = (div * 16.0) as u32;
    }
}

pub(in crate::drivers) unsafe fn hhg_pwm_config_set_wrap(c: *mut pwm_config, wrap: u16) {
    if let Some(c) = unsafe { c.as_mut() } {
        c.top = wrap as u32;
    }
}

pub(in crate::drivers) unsafe fn hhg_pwm_init(_slice_num: u32, _c: *mut pwm_config, _start: bool) {}

pub(in crate::drivers) unsafe fn hhg_pwm_set_gpio_level(gpio: u32, level: u16) {
    with_pin(gpio, |pin| pin.pwm_level = level);
}

pub(in crate::drivers) unsafe fn hhg_gpio_set_irq_enabled_with_callback(gpio: u32, events: u32, enabled: bool, callback: extern "C" fn()) {
    with_pin(gpio, |pin| {
        pin.callback = Some(callback);
        if enabled {
            pin.irq_events |= events;
        } else {
            pin.irq_events &= !events;
        }
    });
}

pub(in crate::drivers) unsafe fn hhg_gpio_set_irq_enabled(gpio: u32, events: u32, enabled: bool) {
    with_pin(gpio, |pin| {
        if enabled {
            pin.irq_events |= events;
        } else {
            pin.irq_events &= !events;
        }
    });
}

pub(in crate::drivers) unsafe fn hhg_adc_init() {}

pub(in crate::drivers) unsafe fn hhg_adc_set_temp_sensor_enabled(_enable: bool) {}

pub(in crate::drivers) unsafe fn hhg_adc_select_input(input: c_uint) {
    ADC_INPUT.store(input, Ordering::Relaxed);
}

pub(in crate::drivers) unsafe fn hhg_adc_read() -> u16 {
    if ADC_INPUT.load(Ordering::Relaxed) == ADC_TEMP_CHANNEL {
        ADC_TEMP_27C
    } else {
        ADC_FLOATING
    }
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_gpio_put(wl_gpio: u32, value: bool) {
    if let Some(level) = lock(&WL_GPIOS).get_mut(wl_gpio as usize) {
        *level = value;
    }
}

pub(in crate::drivers) unsafe fn hhg_cyw43_arch_gpio_get(wl_gpio: u32) -> bool {
    lock(&WL_GPIOS).get(wl_gpio as usize).copied().unwrap_or(false)
}
