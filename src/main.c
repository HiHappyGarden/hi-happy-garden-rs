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

#include <string.h>
#include "mbedtls/aes.h"

extern void start(void);

int main() {
    stdio_init_all();

    unsigned char key[32] = "0123456789abcdef0123456789abcdef";  // Chiave AES-256
    unsigned char iv[16] = "abcdefghijklmnop";   // IV originale
    unsigned char plaintext[] = "Hello, mbed TLS!";
    unsigned char ciphertext[16];
    unsigned char decrypted[16 + 1];
    unsigned char iv_for_decrypt[16];  // Copia dell'IV per decifratura



    mbedtls_aes_context aes;

    // Inizializza contesto AES
    mbedtls_aes_init(&aes);

    // Salva una copia dell'IV per la decifratura
    memcpy(iv_for_decrypt, iv, 16);

    // Cifra
    mbedtls_aes_setkey_enc(&aes, key, 256);
    mbedtls_aes_crypt_cbc(&aes, MBEDTLS_AES_ENCRYPT, 16, iv, plaintext, ciphertext);

    // Decifra (usa la copia dell'IV originale)
    mbedtls_aes_setkey_dec(&aes, key, 256);
    mbedtls_aes_crypt_cbc(&aes, MBEDTLS_AES_DECRYPT, 16, iv_for_decrypt, ciphertext, decrypted);

decrypted[16] = '\0';  // Aggiungi terminatore di stringa

    // Stampa risultati
    printf("Plaintext:  %s\n", plaintext);
    printf("Ciphertext: ");
    for (int i = 0; i < 16; i++) printf("%02X", ciphertext[i]);
    printf("\n");
    printf("Decrypted:  %s\n", decrypted);

    // Pulizia
    mbedtls_aes_free(&aes);

    printf("===================================\r\n");
    printf("=== Hi Happy Garden RS %s ======\r\n", HHG_VER);
    printf("===================================\r\n\r\n");

    start();

    panic_unsupported();

    return EXIT_SUCCESS;
}
