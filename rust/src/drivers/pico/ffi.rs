


#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::ffi::c_uint;

#[repr(C)]
pub struct pwm_config {
    pub csr: u32,
    pub div: u32,
    pub top: u32,
}

pub(super) const GPIO_OUT: bool = true;
pub(super) const GPIO_IN: bool = false;  

#[repr(u32)]
#[derive(Clone, Copy)]
pub(super) enum gpio_function_t {
    GPIO_FUNC_HSTX = 0,
    GPIO_FUNC_SPI = 1,
    GPIO_FUNC_UART = 2,
    GPIO_FUNC_I2C = 3,
    GPIO_FUNC_PWM = 4,
    GPIO_FUNC_SIO = 5,
    GPIO_FUNC_PIO0 = 6,
    GPIO_FUNC_PIO1 = 7,
    GPIO_FUNC_PIO2 = 8,
    GPIO_FUNC_GPCK = 9,
    // GPIO_FUNC_XIP_CS1 = 9,
    // GPIO_FUNC_CORESIGHT_TRACE = 9,
    GPIO_FUNC_USB = 10,
    GPIO_FUNC_UART_AUX = 11,
    GPIO_FUNC_NULL = 0x1f,
}

impl gpio_function_t {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub(super) enum gpio_irq_level {
    GPIO_IRQ_LEVEL_LOW = 0x1,  
    GPIO_IRQ_LEVEL_HIGH = 0x2, 
    GPIO_IRQ_EDGE_FALL = 0x4,  
    GPIO_IRQ_EDGE_RISE = 0x8  
}


#[repr(C)]
#[derive(Clone, Copy)]
pub enum uart_parity_t {
    UART_PARITY_NONE = 0,
    UART_PARITY_EVEN = 1,
    UART_PARITY_ODD = 2,
}

pub type irq_handler_t = unsafe extern "C" fn();

unsafe extern "C" {
    pub(super) fn hhg_gpio_init(gpio: u32);
    pub(super) fn hhg_gpio_set_dir(gpio: u32, out: bool);
    pub(super) fn hhg_gpio_put(gpio: u32, value: bool);
    pub(super) fn hhg_gpio_get(gpio: u32) -> bool;
    pub(super) fn hhg_gpio_pull_up(gpio: u32);
    pub(super) fn hhg_gpio_pull_down(gpio: u32);
    pub(super) fn hhg_gpio_disable_pulls(gpio: u32);
    pub(super) fn hhg_gpio_set_function(gpio: u32, fn_: u32);
    pub(super) fn hhg_pwm_gpio_to_slice_num(gpio: u32) -> u32;
    pub(super) fn hhg_pwm_get_default_config() -> pwm_config;
    pub(super) fn hhg_pwm_config_set_clkdiv(c: *mut pwm_config, div: f32);
    pub(super) fn hhg_pwm_init(slice_num: u32, c: *mut pwm_config, start: bool);
    pub(super) fn hhg_pwm_set_gpio_level(gpio: u32, level: u16);
    pub(super) fn hhg_gpio_set_irq_enabled_with_callback(gpio: u32, events: u32, enabled: bool, callback: extern "C" fn());
    pub(super) fn hhg_gpio_set_irq_enabled(gpio: u32, events: u32, enabled: bool);

    pub(super) fn hhg_uart_init(baudrate: c_uint) -> c_uint;
    pub(super) fn hhg_uart_deinit();
    pub(super) fn hhg_uart_set_hw_flow(cts: bool, rts: bool);
    pub(super) fn hhg_uart_set_format(data_bits: c_uint, stop_bits: c_uint, parity: uart_parity_t);
    pub(super) fn hhg_uart_irq_set_exclusive_handler(handler: irq_handler_t);
    pub(super) fn hhg_uart_irq_set_enabled(enabled: bool);
    pub(super) fn hhg_uart_set_irq_enables(rx_en: bool, tx_en: bool);
    pub(super) fn hhg_uart_is_readable() -> bool;
    pub(super) fn hhg_uart_getc() -> u8;
    pub(super) fn hhg_uart_putc(c: u8);
}   
