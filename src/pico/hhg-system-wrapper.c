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


#include <sys/types.h>
#include <string.h>
#include <pico/unique_id.h>
#include <pico/sha256.h>

extern void * pvPortMalloc( size_t xWantedSize );
extern void vPortFree( void * pv );

void hhg_get_unique_id(uint8_t* id_buffer) {
    pico_unique_board_id_t board_id;
    pico_get_unique_board_id(&board_id);
    memcpy(id_buffer, board_id.id, PICO_UNIQUE_BOARD_ID_SIZE_BYTES);
}

int hhg_pico_sha256_start_blocking(void **state, bool use_dma) {
    *state = pvPortMalloc(sizeof(pico_sha256_state_t));
    return pico_sha256_start_blocking((pico_sha256_state_t *)state, SHA256_BIG_ENDIAN, use_dma);
}

void hhg_pico_sha256_update_blocking(void *state, const uint8_t *data, size_t data_size_bytes) {
    pico_sha256_update((pico_sha256_state_t *)state, data, data_size_bytes);
}

void hhg_pico_sha256_finish(void *state, uint8_t out[SHA256_RESULT_BYTES]) {
    sha256_result_t result;
    pico_sha256_finish((pico_sha256_state_t *)state, &result);
    memcpy(out, result.bytes, SHA256_RESULT_BYTES);
    pico_sha256_cleanup((pico_sha256_state_t *)state);
    vPortFree(state);
}