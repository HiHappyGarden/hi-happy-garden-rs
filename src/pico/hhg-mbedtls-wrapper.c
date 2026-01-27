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

#include <stdlib.h>
#include <string.h>
#include <mbedtls/aes.h>
#include "FreeRTOS.h"

extern void * pvPortMalloc( size_t xWantedSize );
extern void vPortFree( void * pv );

void* hhg_mbedtls_aes_init(void) {
    mbedtls_aes_context* aes = pvPortMalloc(sizeof(mbedtls_aes_context));
    if (aes == NULL) {
        return NULL;
    }
    mbedtls_aes_init(aes);
    return (void*)aes;
}



int hhg_mbedtls_aes_setkey_enc(void* aes, const unsigned char* key, unsigned int keybits) {
    return mbedtls_aes_setkey_enc((mbedtls_aes_context*)aes, key, keybits);
}


int hhg_mbedtls_aes_crypt_cbc(void* aes, int mode, size_t length, unsigned char* iv, const unsigned char* input, unsigned char* output) {
    return mbedtls_aes_crypt_cbc((mbedtls_aes_context*)aes, mode, length, iv, input, output);
}


int hhg_mbedtls_aes_setkey_dec(void* aes, const unsigned char* key, unsigned int keybits) {
    return mbedtls_aes_setkey_dec((mbedtls_aes_context*)aes, key, keybits);
}

void hhg_mbedtls_aes_free(void* aes) {
    mbedtls_aes_free((mbedtls_aes_context*)aes);
    vPortFree((mbedtls_aes_context*)aes);
}