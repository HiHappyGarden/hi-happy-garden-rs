#include <stdio.h>
#include <stdlib.h>
#include <pico/stdlib.h>
#include <pico/stdio.h>
#include <FreeRTOS.h>
#include <task.h>

#define MY_TASK_PRIORITY  2

extern uint64_t app_add(uint64_t left, uint64_t right);

extern void app_main(void);
extern void hardware_main(void);
extern void hardware_start_os(void);

static void my_task(void *data);

int main()
{
    stdio_init_all();

    hardware_main();

    app_main();

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

    xTaskCreate(my_task, "application_task", configMINIMAL_STACK_SIZE, NULL, MY_TASK_PRIORITY, NULL);

    hardware_start_os();

    // we should never return from FreeRTOS
    panic_unsupported();

    return EXIT_SUCCESS;
}

void my_task(void *data) {
    (void)data; // unused parameter

    printf("user task started\n");

    uint64_t a = 0;
    for (;;) {
        // Do something interesting here
        a = app_add(a, 1);
        printf("Hello, world! count%lld\n", a);
        vTaskDelay(pdMS_TO_TICKS(1000));
    }
    // Do not let a task procedure return
    vTaskDelete(NULL);
}