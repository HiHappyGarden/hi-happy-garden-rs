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
