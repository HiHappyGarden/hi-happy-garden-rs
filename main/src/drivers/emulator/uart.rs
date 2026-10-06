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

//! UART0, the AT command port, on the terminal.
//!
//! Twin of `src/pico/hhg-uart-wrapper.c`. TX goes to stdout, where the
//! firmware log already is (on the board both share UART0 too). RX comes
//! from stdin: a reader thread stands in for the UART0 interrupt and calls
//! the handler the firmware registered for every byte received.

use core::ffi::c_uint;
use core::sync::atomic::{AtomicBool, Ordering};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Mutex, Once};

use crate::drivers::pico::ffi::uart_parity;

use super::lock;

static RX_FIFO: Mutex<VecDeque<u8>> = Mutex::new(VecDeque::new());

static IRQ_HANDLER: Mutex<Option<unsafe extern "C" fn()>> = Mutex::new(None);

static IRQ_ENABLED: AtomicBool = AtomicBool::new(false);

static RX_IRQ_ENABLED: AtomicBool = AtomicBool::new(false);

static READER: Once = Once::new();

fn start_reader() {
    READER.call_once(|| {
        let spawned = std::thread::Builder::new()
            .name("emu_uart_rx".into())
            .spawn(|| {
                let mut stdin = std::io::stdin();
                let mut byte = [0u8; 1];
                // EOF (stdin closed or redirected from /dev/null, as in CI)
                // ends the reader: the line just stays silent, like a
                // disconnected cable
                while let Ok(1) = stdin.read(&mut byte) {
                    lock(&RX_FIFO).push_back(byte[0]);
                    raise_rx_irq();
                }
            });
        if let Err(e) = spawned {
            std::eprintln!("emulator: UART RX reader not started: {e}");
        }
    });
}

/// Runs the handler while there are bytes it is consuming, like the RX
/// interrupt stays asserted until the FIFO is drained.
fn raise_rx_irq() {
    loop {
        if !IRQ_ENABLED.load(Ordering::Acquire) || !RX_IRQ_ENABLED.load(Ordering::Acquire) {
            return;
        }
        let Some(handler) = *lock(&IRQ_HANDLER) else {
            return;
        };

        let pending = lock(&RX_FIFO).len();
        if pending == 0 {
            return;
        }

        unsafe { handler() };

        // A handler that does not read would otherwise spin here forever
        if lock(&RX_FIFO).len() >= pending {
            return;
        }
    }
}

pub(in crate::drivers) unsafe fn hhg_uart_init(baudrate: c_uint) -> c_uint {
    start_reader();
    baudrate
}

pub(in crate::drivers) unsafe fn hhg_uart_deinit() {
    IRQ_ENABLED.store(false, Ordering::Release);
    lock(&RX_FIFO).clear();
}

pub(in crate::drivers) unsafe fn hhg_uart_set_hw_flow(_cts: bool, _rts: bool) {}

pub(in crate::drivers) unsafe fn hhg_uart_set_format(_data_bits: c_uint, _stop_bits: c_uint, _parity: uart_parity) {}

pub(in crate::drivers) unsafe fn hhg_uart_set_fifo_enabled(_enabled: bool) {}

pub(in crate::drivers) unsafe fn hhg_uart_irq_set_exclusive_handler(handler: unsafe extern "C" fn()) {
    *lock(&IRQ_HANDLER) = Some(handler);
}

pub(in crate::drivers) unsafe fn hhg_uart_irq_set_enabled(enabled: bool) {
    IRQ_ENABLED.store(enabled, Ordering::Release);
    if enabled {
        // Bytes typed before the firmware was ready are delivered now
        raise_rx_irq();
    }
}

pub(in crate::drivers) unsafe fn hhg_uart_irq_set_high_priority() {}

pub(in crate::drivers) unsafe fn hhg_uart_set_irq_enables(rx_en: bool, _tx_en: bool) {
    RX_IRQ_ENABLED.store(rx_en, Ordering::Release);
}

pub(in crate::drivers) unsafe fn hhg_uart_clear_irq() {}

pub(in crate::drivers) unsafe fn hhg_uart_is_readable() -> bool {
    !lock(&RX_FIFO).is_empty()
}

pub(in crate::drivers) unsafe fn hhg_uart_getc() -> u8 {
    lock(&RX_FIFO).pop_front().unwrap_or(0)
}

pub(in crate::drivers) unsafe fn hhg_uart_read(dst: *mut u8, len: usize) -> usize {
    if dst.is_null() {
        return 0;
    }
    let mut fifo = lock(&RX_FIFO);
    let count = len.min(fifo.len());
    for (i, byte) in fifo.drain(..count).enumerate() {
        unsafe { dst.add(i).write(byte) };
    }
    count
}

pub(in crate::drivers) unsafe fn hhg_uart_putc(c: u8) {
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(&[c]);
    // Written byte by byte like the real port, flushed at each line end so
    // a reply shows up as soon as it is complete
    if c == b'\n' || c == b'\r' {
        let _ = stdout.flush();
    }
}
