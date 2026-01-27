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


#include <stdio.h>
#include <string.h>

#include <pico/stdlib.h>
#include <pico/unique_id.h>

void hhg_get_unique_id(uint8_t* id_buffer) {
    pico_unique_board_id_t board_id;
    pico_get_unique_board_id(&board_id);
    memcpy(id_buffer, board_id.id, PICO_UNIQUE_BOARD_ID_SIZE_BYTES);
}