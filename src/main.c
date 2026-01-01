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

#include "hhg-config.h"

#include <stdio.h>
#include <stdlib.h>
#include <pico/stdlib.h>
#include <pico/stdio.h>
#include <hardware/clocks.h>
#include <hardware/gpio.h>
#include <portmacro.h>



extern void start(void);

int main()
{
    stdio_init_all();


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

    // we should never return from FreeRTOS
    panic_unsupported();

    return EXIT_SUCCESS;
}
