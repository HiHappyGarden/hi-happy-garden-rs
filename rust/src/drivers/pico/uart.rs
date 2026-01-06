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

pub(crate) mod ffi {
    #![allow(non_camel_case_types)]
    #![allow(dead_code)]

    use core::ffi::{c_uint, c_void};

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub enum uart_parity_t {
        UART_PARITY_NONE = 0,
        UART_PARITY_EVEN = 1,
        UART_PARITY_ODD = 2,
    }

    pub type irq_handler_t = unsafe extern "C" fn();

    unsafe extern "C" {
        pub fn hhg_uart_init(baudrate: c_uint) -> c_uint;

        pub fn hhg_uart_deinit();

        pub fn hhg_uart_set_hw_flow(cts: bool, rts: bool);

        pub fn hhg_uart_set_format(data_bits: c_uint, stop_bits: c_uint, parity: uart_parity_t);

        pub fn hhg_uart_irq_set_exclusive_handler(handler: irq_handler_t);

        pub fn hhg_uart_irq_set_enabled(enabled: bool);

        pub fn hhg_uart_set_irq_enables(rx_en: bool, tx_en: bool);

        pub fn hhg_uart_is_readable() -> bool;

        pub fn hhg_uart_getc() -> u8;

        pub fn hhg_uart_putc(c: u8);
    }
}


use core::ptr::null_mut;

use osal_rs::os::MutexFn;
use osal_rs::utils::{Bytes, Error, Result};

use crate::drivers::uart::{UartConfig, UartDataBits, UartFlowControl, UartFn, UartParity, UartStopBits};
use crate::drivers::pico::uart::ffi::{hhg_uart_deinit, hhg_uart_getc, hhg_uart_init, hhg_uart_irq_set_enabled, hhg_uart_irq_set_exclusive_handler, hhg_uart_is_readable, hhg_uart_putc, hhg_uart_set_irq_enables};


pub static mut UART_FN: UartFn = UartFn {
    init,
    transmit,
    receive: None,
    deinit,
};

pub(super) fn get_uart_config() -> UartConfig {
    UartConfig {
        name : &"Uart",
        base: null_mut(),
        baudrate: 115200,
        data_bits: UartDataBits::Eight,
        stop_bits: UartStopBits::One,
        parity: UartParity::None,
        flow_control: UartFlowControl::None,
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "C" fn uart_isr() {

    while hhg_uart_is_readable() {
        let byte = hhg_uart_getc();       

        if let Some(receiver) = &(*&raw const UART_FN.receive) {
            if let Ok(mut receiver) = receiver.lock() {
                receiver.set_source(Bytes::new_by_str("UART"));
                receiver.on_receive(&[byte]); 
            }
        }
    }
}

fn init(config: &UartConfig) -> Result<()> {
    unsafe {
        if hhg_uart_init(config.baudrate) != 0 {
            return Err(Error::Unhandled("Bad baud rate generator value."))
        }

        hhg_uart_irq_set_exclusive_handler(uart_isr);

        hhg_uart_irq_set_enabled(true);

        hhg_uart_set_irq_enables(true, false);
    }

    Ok(())
}

fn transmit(data: &[u8]) -> usize {
    unsafe {
        for &byte in data {
            hhg_uart_putc(byte);
        }
    }
    data.len()
}

fn deinit(_: &UartConfig) -> Result<()> {
    unsafe {
        hhg_uart_irq_set_enabled(false);
        hhg_uart_deinit();
    }
    Ok(())
}