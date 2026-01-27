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

use core::ffi::c_uint;
use core::ptr::null_mut;

use osal_rs::os::{MutexFn, QueueFn};
use osal_rs::utils::{Error, Result};

use crate::drivers::uart::{UartConfig, UartDataBits, UartFlowControl, UartFn, UartParity, UartStopBits};
use crate::drivers::pico::ffi::{gpio_function_type, hhg_gpio_set_function, hhg_uart_deinit, hhg_uart_getc, hhg_uart_init, hhg_uart_irq_set_enabled, hhg_uart_irq_set_exclusive_handler, hhg_uart_is_readable, hhg_uart_putc, hhg_uart_set_format, hhg_uart_set_hw_flow, hhg_uart_set_irq_enables, uart_parity};

// const APP_TAG: &str = "PicoUart";
const TX_PIN: u32 = 0;
const RX_PIN: u32 = 1;

pub static mut UART_FN: UartFn = UartFn {
    init,
    transmit,
    receive: None,
    deinit,
};

pub static mut UART_CONFIG: UartConfig = UartConfig {
    name : &"Uart",
    base: null_mut(),
    baudrate: 115200,
    data_bits: UartDataBits::Eight,
    stop_bits: UartStopBits::One,
    parity: UartParity::None,
    flow_control: UartFlowControl::None,
};

#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "C" fn uart_isr() {

    while hhg_uart_is_readable() {
        let byte = hhg_uart_getc();       

        if let Some(receiver) = *&raw const UART_FN.receive {
            if let Err(_) = receiver.post_from_isr(&[byte]) {
                
            }
        }
    }
}

fn init(config: &UartConfig) -> Result<()> {
    unsafe {

        hhg_gpio_set_function(TX_PIN, gpio_function_type::GPIO_FUNC_UART as u32);

        hhg_gpio_set_function(RX_PIN, gpio_function_type::GPIO_FUNC_UART as u32);

        let UartConfig {
            data_bits,
            stop_bits,
            parity,
            flow_control,
            ..
        } = config;

        let data_bits = match data_bits {
            UartDataBits::Five => 5,
            UartDataBits::Six => 6,
            UartDataBits::Seven => 7,
            UartDataBits::Eight => 8,
            UartDataBits::Nine => 9,
        } as c_uint;
        
        let stop_bits = match stop_bits {
            UartStopBits::One => 1,
            UartStopBits::Two => 2,
            _ => return Err(Error::InvalidType),
        } as c_uint;

        let parity = match parity {
            UartParity::None => uart_parity::UART_PARITY_NONE,
            UartParity::Even => uart_parity::UART_PARITY_EVEN,
            UartParity::Odd => uart_parity::UART_PARITY_ODD,
        };
        
        hhg_uart_set_format(data_bits, stop_bits, parity);

        if *flow_control == UartFlowControl::RtsCts {
            hhg_uart_set_hw_flow(true, true);
        }

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