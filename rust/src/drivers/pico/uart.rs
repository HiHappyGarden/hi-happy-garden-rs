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

use core::ptr::null_mut;

use osal_rs::os::MutexFn;
use osal_rs::utils::{Bytes, Error, Result};

use crate::drivers::uart::{UartConfig, UartDataBits, UartFlowControl, UartFn, UartParity, UartStopBits};
use crate::drivers::pico::ffi::{gpio_function_t, hhg_gpio_set_function, hhg_uart_deinit, hhg_uart_getc, hhg_uart_init, hhg_uart_irq_set_enabled, hhg_uart_irq_set_exclusive_handler, hhg_uart_is_readable, hhg_uart_putc, hhg_uart_set_irq_enables};

const TX_PIN: u32 = 0;
const RX_PIN: u32 = 1;

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

        hhg_gpio_set_function(TX_PIN, gpio_function_t::GPIO_FUNC_UART.as_u32());
        hhg_gpio_set_function(RX_PIN, gpio_function_t::GPIO_FUNC_UART.as_u32());

        hhg_uart_init(config.baudrate);

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