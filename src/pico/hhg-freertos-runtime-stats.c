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

#include <hardware/timer.h>

/**
 * @brief Configure timer for FreeRTOS runtime statistics
 * 
 * Uses the RP2040/RP2350 hardware timer which runs at 1MHz.
 * This is much faster than the typical FreeRTOS tick (1kHz),
 * providing good resolution for runtime statistics.
 */
void vConfigureTimerForRunTimeStats(void)
{
    // No initialization needed - the hardware timer is always running
    // and provides microsecond resolution (1MHz)
}

/**
 * @brief Get current runtime counter value
 * 
 * @return uint32_t Current timer value in microseconds
 */
uint32_t ulGetRunTimeCounterValue(void)
{
    // Return the current 64-bit microsecond timer as 32-bit
    // This will wrap after ~71 minutes, which is acceptable for runtime stats
    return (uint32_t)time_us_64();
}
