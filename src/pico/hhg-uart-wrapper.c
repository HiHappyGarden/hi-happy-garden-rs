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

#include "pico/types.h"
#include "pico/stdlib.h"
#include "hardware/irq.h"
#include "hardware/uart.h"
#include "hardware/regs/uart.h"



uint hhg_uart_init(uint baudrate) {
    return uart_init(uart0, baudrate);
}

void hhg_uart_deinit(void) {
    uart_deinit(uart0);
}

void hhg_uart_set_hw_flow(bool cts, bool rts) {
    uart_set_hw_flow(uart0, cts, rts);
}

void hhg_uart_set_format(uint data_bits, uint stop_bits, uart_parity_t parity) {
    uart_set_format(uart0, data_bits, stop_bits, parity);
}

void hhg_uart_set_fifo_enabled(bool enabled) {
    uart_set_fifo_enabled(uart0, enabled);
}


void hhg_uart_irq_set_exclusive_handler(irq_handler_t handler) {
    irq_set_exclusive_handler(UART0_IRQ, handler);
}

void hhg_uart_irq_set_enabled(bool enabled) {
    irq_set_enabled(UART0_IRQ, enabled);
}

void hhg_uart_irq_set_high_priority(void) {
    irq_set_priority(UART0_IRQ, PICO_DEFAULT_IRQ_PRIORITY);
}

void hhg_uart_set_irq_enables(bool rx_en, bool tx_en) {
    uart_set_irq_enables(uart0, rx_en, tx_en);
}

void hhg_uart_clear_irq(void) {
    uart_get_hw(uart0)->icr =
        UART_UARTICR_RXIC_BITS |
        UART_UARTICR_RTIC_BITS |
        UART_UARTICR_OEIC_BITS |
        UART_UARTICR_BEIC_BITS |
        UART_UARTICR_PEIC_BITS |
        UART_UARTICR_FEIC_BITS;
}

bool hhg_uart_is_readable(void) {
    return uart_is_readable(uart0);
}

uint8_t hhg_uart_getc(void) {
    return uart_getc(uart0);
}


size_t hhg_uart_read(uint8_t *dst, size_t len) {
    size_t count = 0;

    while (count < len && uart_is_readable(uart0)) {
        dst[count++] = uart_getc(uart0);
    }

    return count;
}


void hhg_uart_putc(uint8_t c) {
    uart_putc(uart0, c);
}