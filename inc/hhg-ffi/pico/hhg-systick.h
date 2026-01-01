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

#ifndef SYSTICK_DEBUG_H
#define SYSTICK_DEBUG_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Call this function from a FreeRTOS task to check SysTick status
void print_systick_status(void);

uint32_t get_g_setup_called(void);

#ifdef __cplusplus
}
#endif

#endif // SYSTICK_DEBUG_H
