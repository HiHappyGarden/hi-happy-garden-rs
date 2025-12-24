#include "hhg-config.h"

#include <stdio.h>
#include <stdlib.h>
#include <pico/stdlib.h>
#include <pico/stdio.h>
#include <hardware/clocks.h>
#include <hardware/gpio.h>



extern void start(void);
extern void start_os(void);

int main()
{
    stdio_init_all();

    // // Diagnostic: Print system clock frequency before starting FreeRTOS
    // uint32_t sys_clock_hz = clock_get_hz(clk_sys);
    // printf("System clock frequency: %u Hz\n", sys_clock_hz);
    
    // if (sys_clock_hz == 0) 
    // {
    //     printf("ERROR: System clock not initialized!\n");
    //     panic("System clock is 0");
    // }
    
    // // Expected SysTick reload value
    // uint32_t expected_reload = (sys_clock_hz / 1000) - 1;  // configTICK_RATE_HZ = 1000
    // printf("Expected SysTick reload value: %u\n", expected_reload);

    printf("===================================\r\n");
    printf("=== Hi Happy Garden RS %s ======\r\n", HHG_VER);
    printf("===================================\r\n\r\n");

    start();

    // // Initialise the Wi-Fi chip
    // if (cyw43_arch_init()) {
    //     printf("Wi-Fi init failed\n");
    //     return -1;
    // }

    // // Enable wifi station
    // cyw43_arch_enable_sta_mode();

    // printf("Connecting to Wi-Fi...\n");
    // if (cyw43_arch_wifi_connect_timeout_ms("Vodafone-salsi.local", "s4ls3tt4", CYW43_AUTH_WPA2_AES_PSK, 30000)) {
    //     printf("failed to connect.\n");
    // } else {
    //     printf("Connected.\n");
    //     // Read the ip address in a human readable way
    //     uint8_t *ip_address = (uint8_t*)&(cyw43_state.netif[0].ip_addr.addr);
    //     printf("IP address %d.%d.%d.%d\n", ip_address[0], ip_address[1], ip_address[2], ip_address[3]);
    // }


    start_os();

    // we should never return from FreeRTOS
    panic_unsupported();

    return EXIT_SUCCESS;
}
