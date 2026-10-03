/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Crypto tests on the real accelerators: SHA256 on the RP2350 hardware block,
//! AES-CBC through mbedtls. Known-answer vectors come from FIPS 180-2 and
//! NIST SP 800-38A (F.2.1 / F.2.5), so a wrong key schedule or byte order
//! shows up as a mismatch, not just as a failed roundtrip.

use alloc::vec;

use osal_rs::utils::{Result, hex_to_bytes};

use super::{Encrypt, EncryptGeneric, SHA256_RESULT_BYTES};
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::state::Initializable;

const TAG: &str = "EncryptTests";

const NIST_IV: [u8; 16] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f];
const NIST_PLAIN: &str = "6bc1bee22e409f96e93d7e117393172aae2d8a571e03ac9c9eb76fac45af8e51";

/// Runs `test` on an initialised AES context and always frees it afterwards.
fn with_aes<const KEY_SIZE: usize>(key: &[u8; KEY_SIZE], iv: &[u8; 16], test: impl FnOnce(&Encrypt<KEY_SIZE, 16>) -> Result<()>) -> Result<()> {
    let mut encrypt = Encrypt::<KEY_SIZE, 16>::shared(key, iv)?;
    encrypt.init()?;
    let ret = test(&encrypt);
    encrypt.drop();
    ret
}

fn test_sha256_known_answers() -> Result<()> {
    let empty = EncryptGeneric::get_sha256(b"")?;
    test_assert_eq!(empty.as_str(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

    let abc = EncryptGeneric::get_sha256(b"abc")?;
    test_assert_eq!(abc.as_str(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");

    // Longer than one 64-byte block
    let long = EncryptGeneric::get_sha256(&vec![b'a'; 1_000])?;
    test_assert_eq!(long.as_str(), "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3");
    test_assert_eq!(long.len(), SHA256_RESULT_BYTES * 2);
    Ok(())
}

fn test_sha256_repeatable() -> Result<()> {
    // The hardware block is shared: back-to-back hashes must not leak state
    for _ in 0..10 {
        test_assert_eq!(EncryptGeneric::get_sha256(b"abc")?.as_str(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
    test_assert!(EncryptGeneric::get_sha256(b"abc")? != EncryptGeneric::get_sha256(b"abd")?);
    Ok(())
}

fn test_aes128_cbc_known_answer() -> Result<()> {
    let key: [u8; 16] = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
    with_aes(&key, &NIST_IV, |aes| {
        let plain = hex_to_bytes(NIST_PLAIN)?;
        let cipher = aes.aes_encrypt(&plain)?;
        test_assert_eq!(cipher, hex_to_bytes("7649abac8119b246cee98e9b12e9197d5086cb9b507219ee95db113a917678b2")?);
        test_assert_eq!(aes.aes_decrypt(&cipher)?, plain);
        Ok(())
    })
}

fn test_aes256_cbc_known_answer() -> Result<()> {
    let key: [u8; 32] = [
        0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81,
        0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4,
    ];
    with_aes(&key, &NIST_IV, |aes| {
        let plain = hex_to_bytes(NIST_PLAIN)?;
        let cipher = aes.aes_encrypt(&plain)?;
        test_assert_eq!(cipher, hex_to_bytes("f58c4c04d6e5f1ba779eabfb5f7bfbd69cfc4e967edb808d679f777bc6702c7d")?);
        test_assert_eq!(aes.aes_decrypt(&cipher)?, plain);
        Ok(())
    })
}

fn test_aes_zero_padding() -> Result<()> {
    let key = [0x11u8; 32];
    with_aes(&key, &NIST_IV, |aes| {
        let plain = b"hello garden";
        let cipher = aes.aes_encrypt(plain)?;
        // Zero padded to the next 16-byte block, no PKCS#7
        test_assert_eq!(cipher.len(), 16);
        let decrypted = aes.aes_decrypt(&cipher)?;
        test_assert_eq!(&decrypted[..plain.len()], &plain[..]);
        test_assert!(decrypted[plain.len()..].iter().all(|&b| b == 0));

        test_assert_eq!(aes.aes_encrypt(&[0u8; 16])?.len(), 16);
        test_assert_eq!(aes.aes_encrypt(&[0u8; 17])?.len(), 32);
        test_assert!(aes.aes_encrypt(&[])?.is_empty());
        Ok(())
    })
}

fn test_aes_iv_and_key_matter() -> Result<()> {
    let plain = [0x42u8; 32];
    let key_a = [0x01u8; 16];
    let key_b = [0x02u8; 16];
    let iv_b = [0xffu8; 16];

    let mut cipher_a = vec![];
    with_aes(&key_a, &NIST_IV, |aes| { cipher_a = aes.aes_encrypt(&plain)?; Ok(()) })?;

    // CBC: equal plaintext blocks must not give equal ciphertext blocks
    test_assert!(cipher_a[..16] != cipher_a[16..]);

    with_aes(&key_b, &NIST_IV, |aes| {
        test_assert!(aes.aes_encrypt(&plain)? != cipher_a, "different key, same ciphertext");
        Ok(())
    })?;
    with_aes(&key_a, &iv_b, |aes| {
        test_assert!(aes.aes_encrypt(&plain)? != cipher_a, "different IV, same ciphertext");
        Ok(())
    })
}

fn test_invalid_sizes() -> Result<()> {
    let key15 = [0u8; 15];
    let key16 = [0u8; 16];
    let iv8 = [0u8; 8];
    test_assert!(Encrypt::<15, 16>::shared(&key15, &NIST_IV).is_err());
    test_assert!(Encrypt::<16, 8>::shared(&key16, &iv8).is_err());
    test_assert!(Encrypt::<16, 16>::shared(&key16, &NIST_IV).is_ok());
    Ok(())
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_sha256_known_answers,
        test_sha256_repeatable,
        test_aes128_cbc_known_answer,
        test_aes256_cbc_known_answer,
        test_aes_zero_padding,
        test_aes_iv_and_key_matter,
        test_invalid_sizes,
    );
}
