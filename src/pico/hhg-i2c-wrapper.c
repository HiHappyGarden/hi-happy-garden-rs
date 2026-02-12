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

#include <pico/types.h>
#include <pico/binary_info.h>
#include <hardware/i2c.h>
#include <hardware/gpio.h>


void* hhg_i2c_instance(uint8_t i2c_num) {
    return I2C_INSTANCE(i2c_num);
}

uint hhg_i2c_init(void *i2c, uint baudrate) {
    if (i2c == NULL) {
        return 0;
    }
    return i2c_init((i2c_inst_t *)i2c, baudrate);
}

void hhg_i2c_init_pins_with_func(void) {
    bi_decl(bi_2pins_with_func(2, 3, GPIO_FUNC_I2C));
}

int hhg_i2c_write_blocking(void *i2c, uint8_t addr, const uint8_t *src, size_t len, bool nostop) {
    return i2c_write_blocking((i2c_inst_t *)i2c, addr, src, len, nostop);
}

int hhg_i2c_read_blocking(void *i2c, uint8_t addr, uint8_t *dst, size_t len, bool nostop) {
    return i2c_read_blocking((i2c_inst_t *)i2c, addr, dst, len, nostop);
}

void hhg_i2c_deinit(void *i2c) {
    i2c_deinit((i2c_inst_t *)i2c);
}
