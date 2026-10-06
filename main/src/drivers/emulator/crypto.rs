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

//! mbedtls AES-CBC on top of the RustCrypto `aes` block cipher.
//!
//! Twin of `src/pico/hhg-mbedtls-wrapper.c`, keeping the mbedtls contract
//! the firmware relies on: the same error codes, the IV updated in place
//! for chaining, and input and output allowed to be the same buffer.

use alloc::boxed::Box;
use core::ffi::c_void;

use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::{Aes128, Aes192, Aes256, Block};

use crate::drivers::pico::ffi::aes_mode;

const AES_BLOCK_SIZE: usize = 16;

// mbedtls/aes.h
const MBEDTLS_ERR_AES_INVALID_KEY_LENGTH: i32 = -0x0020;
const MBEDTLS_ERR_AES_INVALID_INPUT_LENGTH: i32 = -0x0022;
const MBEDTLS_ERR_AES_BAD_INPUT_DATA: i32 = -0x0021;

/// `mbedtls_aes_context`: the expanded key of the last setkey call
enum AesContext {
    Empty,
    Aes128(Aes128),
    Aes192(Aes192),
    Aes256(Aes256),
}

impl AesContext {
    fn new(key: &[u8]) -> Option<Self> {
        Some(match key.len() * 8 {
            128 => Self::Aes128(Aes128::new_from_slice(key).ok()?),
            192 => Self::Aes192(Aes192::new_from_slice(key).ok()?),
            256 => Self::Aes256(Aes256::new_from_slice(key).ok()?),
            _ => return None,
        })
    }

    fn encrypt(&self, block: &mut Block) -> bool {
        match self {
            Self::Empty => return false,
            Self::Aes128(cipher) => cipher.encrypt_block(block),
            Self::Aes192(cipher) => cipher.encrypt_block(block),
            Self::Aes256(cipher) => cipher.encrypt_block(block),
        }
        true
    }

    fn decrypt(&self, block: &mut Block) -> bool {
        match self {
            Self::Empty => return false,
            Self::Aes128(cipher) => cipher.decrypt_block(block),
            Self::Aes192(cipher) => cipher.decrypt_block(block),
            Self::Aes256(cipher) => cipher.decrypt_block(block),
        }
        true
    }
}

pub(in crate::drivers) unsafe fn hhg_mbedtls_aes_init() -> *mut c_void {
    Box::into_raw(Box::new(AesContext::Empty)) as *mut c_void
}

unsafe fn setkey(aes: *mut c_void, key: *const u8, keybits: u32) -> i32 {
    let Some(context) = (unsafe { (aes as *mut AesContext).as_mut() }) else {
        return MBEDTLS_ERR_AES_BAD_INPUT_DATA;
    };
    if key.is_null() || !matches!(keybits, 128 | 192 | 256) {
        return MBEDTLS_ERR_AES_INVALID_KEY_LENGTH;
    }

    let key = unsafe { core::slice::from_raw_parts(key, keybits as usize / 8) };
    match AesContext::new(key) {
        Some(expanded) => {
            *context = expanded;
            0
        }
        None => MBEDTLS_ERR_AES_INVALID_KEY_LENGTH,
    }
}

pub(in crate::drivers) unsafe fn hhg_mbedtls_aes_setkey_enc(aes: *mut c_void, key: *const u8, keybits: u32) -> i32 {
    unsafe { setkey(aes, key, keybits) }
}

pub(in crate::drivers) unsafe fn hhg_mbedtls_aes_setkey_dec(aes: *mut c_void, key: *const u8, keybits: u32) -> i32 {
    unsafe { setkey(aes, key, keybits) }
}

pub(in crate::drivers) unsafe fn hhg_mbedtls_aes_crypt_cbc(aes: *mut c_void, mode: i32, length: usize, iv: *mut u8, input: *const u8, output: *mut u8) -> i32 {
    let Some(context) = (unsafe { (aes as *const AesContext).as_ref() }) else {
        return MBEDTLS_ERR_AES_BAD_INPUT_DATA;
    };
    if mode != aes_mode::AES_ENCRYPT as i32 && mode != aes_mode::AES_DECRYPT as i32 {
        return MBEDTLS_ERR_AES_BAD_INPUT_DATA;
    }
    if !length.is_multiple_of(AES_BLOCK_SIZE) {
        return MBEDTLS_ERR_AES_INVALID_INPUT_LENGTH;
    }
    if length == 0 {
        return 0;
    }
    if iv.is_null() || input.is_null() || output.is_null() {
        return MBEDTLS_ERR_AES_BAD_INPUT_DATA;
    }

    let mut chain = Block::default();
    unsafe { core::ptr::copy_nonoverlapping(iv, chain.as_mut_ptr(), AES_BLOCK_SIZE) };

    for offset in (0..length).step_by(AES_BLOCK_SIZE) {
        // Whole block copied in before anything is written out: input and
        // output may be the very same buffer
        let mut block = Block::default();
        unsafe { core::ptr::copy(input.add(offset), block.as_mut_ptr(), AES_BLOCK_SIZE) };

        let done = if mode == aes_mode::AES_ENCRYPT as i32 {
            block.iter_mut().zip(chain.iter()).for_each(|(byte, iv)| *byte ^= iv);
            let ok = context.encrypt(&mut block);
            chain = block;
            ok
        } else {
            let cipher = block;
            let ok = context.decrypt(&mut block);
            block.iter_mut().zip(chain.iter()).for_each(|(byte, iv)| *byte ^= iv);
            chain = cipher;
            ok
        };
        if !done {
            return MBEDTLS_ERR_AES_BAD_INPUT_DATA;
        }

        unsafe { core::ptr::copy(block.as_ptr(), output.add(offset), AES_BLOCK_SIZE) };
    }

    // mbedtls leaves the last ciphertext block in the IV, ready to chain
    unsafe { core::ptr::copy_nonoverlapping(chain.as_ptr(), iv, AES_BLOCK_SIZE) };
    0
}

pub(in crate::drivers) unsafe fn hhg_mbedtls_aes_free(aes: *mut c_void) {
    if !aes.is_null() {
        drop(unsafe { Box::from_raw(aes as *mut AesContext) });
    }
}
