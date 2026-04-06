/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
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

use core::ffi::c_uint;
use core::ptr::null_mut;

use alloc::boxed::Box;
use alloc::sync::Arc;
use osal_rs::os::types::{BaseType, StackType, TickType};
use osal_rs::os::{System, SystemFn, Thread, ThreadFn, ThreadNotification, ThreadParam};
use osal_rs::utils::{Error, Result};

use crate::drivers::platform::ThreadPriority;
use crate::drivers::uart::{UartConfig, UartDataBits, UartFlowControl, UartFn, UartParity, UartStopBits};
use crate::drivers::pico::ffi::{gpio_function_type, hhg_gpio_set_function, hhg_uart_deinit, hhg_uart_init, hhg_uart_irq_set_enabled, hhg_uart_irq_set_exclusive_handler, hhg_uart_putc, hhg_uart_read, hhg_uart_set_format, hhg_uart_set_hw_flow, hhg_uart_set_irq_enables, uart_parity};
use crate::traits::rx_tx::Source;

const TX_PIN: u32 = 0;
const RX_PIN: u32 = 1;

const RX_CHUNK_SIZE: usize = 64;

static mut RX_THREAD: Option<Thread> = None;
static mut RX_THREAD_RUNNING: bool = false;

const RX_THREAD_NAME: &str = "uart_rx_trd";
const RX_THREAD_STACK_SIZE: StackType = 1_024;

pub static mut UART_FN: UartFn = UartFn {
    init,
    transmit,
    add_listener: None,
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
    hhg_uart_irq_set_enabled(false);

    let mut task_woken: BaseType = 0;
    if let Some(thread) = (*&raw const RX_THREAD).clone() {
        let _ = thread.notify_from_isr(ThreadNotification::Increment, &mut task_woken);
    }

    System::yield_from_isr(task_woken);
}

fn uart_rx_worker(current_thread: Box<dyn ThreadFn>, _: Option<ThreadParam>) -> Result<ThreadParam> {

    let mut buffer = [0u8; RX_CHUNK_SIZE];

    loop {
        if current_thread.wait_notification(0, u32::MAX, TickType::MAX).is_err() {
            continue;
        }

        if unsafe { !RX_THREAD_RUNNING } {
            break;
        }

        loop {
            let received = unsafe { hhg_uart_read(buffer.as_mut_ptr(), buffer.len()) };
            if received == 0 {
                break;
            }

            if let Some(listener) = unsafe { *&raw const UART_FN.add_listener } {
                let _ = (*listener).on_receive(Source::Uart, &buffer[..received]);
            }
        }

        unsafe {
            hhg_uart_irq_set_enabled(true);
        }
    }
    Ok(Arc::new(()))
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
        
        // Initialize UART first
        hhg_uart_init(config.baudrate);
        
        // Then configure format
        hhg_uart_set_format(data_bits, stop_bits, parity);

        if *flow_control == UartFlowControl::RtsCts {
            hhg_uart_set_hw_flow(true, true);
        }

        RX_THREAD_RUNNING = true;

        if (*&raw const RX_THREAD).is_none() {
            let mut thread = Thread::new_with_to_priority(RX_THREAD_NAME, RX_THREAD_STACK_SIZE, ThreadPriority::High);
            let thread = thread.spawn(None, uart_rx_worker)?;
            RX_THREAD = Some(thread);
        }

        hhg_uart_irq_set_exclusive_handler(uart_isr);
        hhg_uart_set_irq_enables(true, false);
        hhg_uart_irq_set_enabled(true);
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
        hhg_uart_set_irq_enables(false, false);
        RX_THREAD_RUNNING = false;
        if let Some(thread) = (*&raw const RX_THREAD).clone() {
            let _ = thread.notify(ThreadNotification::Increment);
        }
        RX_THREAD = None;
        hhg_uart_deinit();
    }
    Ok(())
}