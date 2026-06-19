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


bool hhg_add_repeating_timer_ms(int32_t delay_ms, void (*callback)(void *), void *user_data, void **out) {
    if (out == NULL) {
        return false;
    }

    if (*out) {
        vPortFree(*out);
    }
    *out = pvPortMalloc(sizeof(repeating_timer_t));
    if (*out == NULL) {
        return false;
    }

    return add_repeating_timer_ms(delay_ms, (repeating_timer_callback_t)callback, user_data, (repeating_timer_t *)*out);
}


bool hhg_cancel_repeating_timer(void *timer) {
    if (timer == NULL) {
        return false;
    }
    bool rc = cancel_repeating_timer((repeating_timer_t *)timer);

    vPortFree((repeating_timer_t *)timer);

    return rc;
}