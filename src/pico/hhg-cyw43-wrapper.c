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

#include <pico/cyw43_arch.h>
#include <pico/types.h>

int hhg_cyw43_arch_init_with_country(uint32_t country_code) {
    return cyw43_arch_init_with_country(country_code);
}

void hhg_cyw43_arch_gpio_put(uint wl_gpio, bool value) {
    cyw43_arch_gpio_put(wl_gpio, value);
}

void hhg_cyw43_arch_deinit(void) {
    cyw43_arch_deinit();
}

void hhg_cyw43_arch_enable_sta_mode(void) {
    cyw43_arch_enable_sta_mode();
}

void hhg_cyw43_arch_disable_sta_mode(void) {
    cyw43_arch_disable_sta_mode();
}

int hhg_cyw43_wifi_link_status(int itf) {
    return cyw43_wifi_link_status(&cyw43_state, itf);
}

int hhg_cyw43_arch_wifi_connect(const char *ssid, const char *pw, uint32_t auth) {
    return cyw43_arch_wifi_connect_timeout_ms(ssid, pw, auth, 10000);
}

void hhg_cyw43_arch_poll(void) {
    cyw43_arch_poll();
}

void hhg_cyw43_arch_lwip_begin(void) {
    cyw43_arch_lwip_begin();
}

void hhg_cyw43_arch_lwip_end(void) {
    cyw43_arch_lwip_end();
}
