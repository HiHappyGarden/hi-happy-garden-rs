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

#include "hhg-systick.h"
#include "FreeRTOS.h"
#include "task.h"

#include <stdio.h>
#include <hardware/clocks.h>

// SysTick register definitions (from ARM Cortex-M33)
#define portNVIC_SYSTICK_CTRL_REG           ( *( ( volatile uint32_t * ) 0xe000e010 ) )
#define portNVIC_SYSTICK_LOAD_REG           ( *( ( volatile uint32_t * ) 0xe000e014 ) )
#define portNVIC_SYSTICK_CURRENT_VALUE_REG  ( *( ( volatile uint32_t * ) 0xe000e018 ) )

#define portNVIC_SYSTICK_CLK_BIT            ( 1UL << 2UL )
#define portNVIC_SYSTICK_INT_BIT            ( 1UL << 1UL )
#define portNVIC_SYSTICK_ENABLE_BIT         ( 1UL << 0UL )
#define portNVIC_SYSTICK_COUNT_FLAG_BIT     ( 1UL << 16UL )

// Store configuration values for later display
static uint32_t g_systick_clock_hz = 0;
static uint32_t g_systick_reload_value = 0;
static uint32_t g_setup_called = 0;  // Flag to check if vPortSetupTimerInterrupt was called

// Override the weak function from port.c to add diagnostics
void vPortSetupTimerInterrupt( void )
{
    uint32_t sys_clock_hz = clock_get_hz(clk_sys);
    uint32_t reload_value;
    
    g_setup_called = 1;  // Mark that this function was called
    
    // Store values for later display (can't safely use printf here)
    g_systick_clock_hz = sys_clock_hz;
    
    if (sys_clock_hz == 0) 
    {
        // System clock not initialized - this is a critical error
        g_systick_reload_value = 0;
        return;
    }
    
    reload_value = (sys_clock_hz / configTICK_RATE_HZ) - 1UL;
    g_systick_reload_value = reload_value;
    
    // Stop and reset SysTick (use CLK_BIT, not CLK_BIT_CONFIG)
    portNVIC_SYSTICK_CTRL_REG = portNVIC_SYSTICK_CLK_BIT;
    portNVIC_SYSTICK_CURRENT_VALUE_REG = 0UL;
    
    // Configure SysTick to interrupt at the requested rate
    portNVIC_SYSTICK_LOAD_REG = reload_value;
    portNVIC_SYSTICK_CTRL_REG = portNVIC_SYSTICK_CLK_BIT | 
                                portNVIC_SYSTICK_INT_BIT | 
                                portNVIC_SYSTICK_ENABLE_BIT;
}

// Diagnostic function to check SysTick status (call from a task)
void print_systick_status(void)
{
    printf("=== SysTick Configuration Debug ===\n");
    printf("vPortSetupTimerInterrupt called: %s\n", g_setup_called ? "YES" : "NO");
    printf("System clock: %u Hz\n", g_systick_clock_hz);
    printf("configTICK_RATE_HZ: %u Hz\n", (unsigned int)configTICK_RATE_HZ);
    printf("Calculated reload value: %u (0x%08X)\n", g_systick_reload_value, g_systick_reload_value);
    printf("===================================\n\n");
    
    printf("=== SysTick Status Check ===\n");
    printf("CTRL: 0x%08X ", portNVIC_SYSTICK_CTRL_REG);
    printf("(Enable: %d, TickInt: %d, ClkSource: %d, CountFlag: %d)\n",
           (portNVIC_SYSTICK_CTRL_REG & portNVIC_SYSTICK_ENABLE_BIT) ? 1 : 0,
           (portNVIC_SYSTICK_CTRL_REG & portNVIC_SYSTICK_INT_BIT) ? 1 : 0,
           (portNVIC_SYSTICK_CTRL_REG & portNVIC_SYSTICK_CLK_BIT) ? 1 : 0,
           (portNVIC_SYSTICK_CTRL_REG & portNVIC_SYSTICK_COUNT_FLAG_BIT) ? 1 : 0);
    printf("LOAD: 0x%08X (%u)\n", portNVIC_SYSTICK_LOAD_REG, portNVIC_SYSTICK_LOAD_REG);
    printf("VAL:  0x%08X (%u)\n", portNVIC_SYSTICK_CURRENT_VALUE_REG, portNVIC_SYSTICK_CURRENT_VALUE_REG);
    printf("Tick count: %u\n", (unsigned int)xTaskGetTickCount());
    printf("============================\n\n");
}

uint32_t get_g_setup_called(void)
{
    return g_setup_called;
}
