#include <stdio.h>
#include "pico/stdlib.h"
#include "pico/cyw43_arch.h"

extern "C" uint64_t app_add(uint64_t left, uint64_t right);
extern "C" uint64_t hw_add(uint64_t left, uint64_t right);


int main()
{
    stdio_init_all();

    // Initialise the Wi-Fi chip
    if (cyw43_arch_init()) {
        printf("Wi-Fi init failed\n");
        return -1;
    }

    // Enable wifi station
    cyw43_arch_enable_sta_mode();

    printf("Connecting to Wi-Fi...\n");
    if (cyw43_arch_wifi_connect_timeout_ms("Vodafone-salsi.local", "s4ls3tt4", CYW43_AUTH_WPA2_AES_PSK, 30000)) {
        printf("failed to connect.\n");
    } else {
        printf("Connected.\n");
        // Read the ip address in a human readable way
        uint8_t *ip_address = (uint8_t*)&(cyw43_state.netif[0].ip_addr.addr);
        printf("IP address %d.%d.%d.%d\n", ip_address[0], ip_address[1], ip_address[2], ip_address[3]);
    }

    uint64_t a = 0;
    while (true) {
        a = app_add(a, 1);
        a = hw_add(a, 1);
        printf("Hello, world! count%lld\n", a);
        sleep_ms(1000);
    }
}
