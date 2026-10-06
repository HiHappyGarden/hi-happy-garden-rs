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

#include <stdio.h>
#include "pico/stdlib.h"


extern void * pvPortMalloc( size_t xWantedSize );
extern void vPortFree( void * pv );

// The pico-sdk calls back with the repeating_timer_t and wants a bool back
// (true: keep repeating), Rust registers a plain void (*)(void*): the
// trampoline adapts the two, the handle carries what it needs
typedef struct {
    repeating_timer_t timer;
    void (*callback)(void *);
    void *user_data;
} hhg_repeating_timer_t;

static bool hhg_repeating_timer_trampoline(repeating_timer_t *rt) {
    hhg_repeating_timer_t *handle = (hhg_repeating_timer_t *)rt->user_data;
    handle->callback(handle->user_data);
    return true;
}

bool hhg_cancel_repeating_timer(void *timer);

bool hhg_add_repeating_timer_ms(int32_t delay_ms, void (*callback)(void *), void *user_data, void **out) {
    if (out == NULL || callback == NULL) {
        return false;
    }

    // A timer still running in *out is stopped before its memory is reused
    if (*out) {
        hhg_cancel_repeating_timer(*out);
        *out = NULL;
    }

    hhg_repeating_timer_t *handle = pvPortMalloc(sizeof(hhg_repeating_timer_t));
    if (handle == NULL) {
        return false;
    }
    handle->callback = callback;
    handle->user_data = user_data;

    if (!add_repeating_timer_ms(delay_ms, hhg_repeating_timer_trampoline, handle, &handle->timer)) {
        vPortFree(handle);
        return false;
    }

    *out = handle;
    return true;
}


bool hhg_cancel_repeating_timer(void *timer) {
    if (timer == NULL) {
        return false;
    }
    hhg_repeating_timer_t *handle = (hhg_repeating_timer_t *)timer;
    bool rc = cancel_repeating_timer(&handle->timer);

    vPortFree(handle);

    return rc;
}
