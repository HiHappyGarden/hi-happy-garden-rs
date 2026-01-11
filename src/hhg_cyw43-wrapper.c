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

typedef unsigned int uint;

int hhg_cyw43_arch_init(void) {
    return cyw43_arch_init();
}

void hhg_cyw43_arch_gpio_put(uint wl_gpio, bool value) {
    cyw43_arch_gpio_put(wl_gpio, value);
}