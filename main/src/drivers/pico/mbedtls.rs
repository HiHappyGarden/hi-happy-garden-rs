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

//#![allow(unused)]

use core::ffi::c_void;

use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};
use core::ptr::null_mut;
use osal_rs::utils::{bytes_to_hex_into_slice, Bytes, Error, Result};

use crate::drivers::pico::ffi::{aes_mode, hhg_mbedtls_aes_crypt_cbc, hhg_mbedtls_aes_free, hhg_mbedtls_aes_init, hhg_mbedtls_aes_setkey_enc, hhg_mbedtls_aes_setkey_dec};
use crate::drivers::encrypt::{EncryptFn, SHA256_RESULT_BYTES};
use crate::drivers::plt::ffi::{hhg_pico_sha256_finish, hhg_pico_sha256_start_blocking, hhg_pico_sha256_update_blocking};

pub const ENCRYPT_FN: EncryptFn = EncryptFn {
    init,
    aes_encrypt,
    aes_decrypt,
    get_sha256,
    drop,
};

fn enc_dec(handler: *mut c_void, mode: aes_mode, key: &[u8], iv: &[u8], buffer: &[u8]) -> Result<Vec<u8>> {
    
    let key_bits: u32 =  if key.len() == 16 {
        128
    } else if key.len() == 24 {
        192
    } else if key.len() == 32 {
        256
    } else {
        return Err(Error::Unhandled("Invalid key size. Must be 16, 24, or 32 bytes."));
    };


    let padded_len: usize = if mode == aes_mode::AES_ENCRYPT { 
        (buffer.len() + 15) & !15usize
    } else { 
        buffer.len()
    };

    let mut output = Vec::<u8>::with_capacity(padded_len); 
    output.resize(padded_len, 0u8);

    // Copy of IV because it is modified during CBC operation
    let mut iv_copy = iv.to_vec();

    unsafe { 
        let ret = if mode == aes_mode::AES_ENCRYPT {
            hhg_mbedtls_aes_setkey_enc(handler, key.as_ptr(), key_bits)
        } else {
            hhg_mbedtls_aes_setkey_dec(handler, key.as_ptr(), key_bits)
        };
        
        if ret != 0 {
            return Err(Error::ReturnWithCode(ret));
        }


        let (input_ptr, output_ptr) = if mode == aes_mode::AES_ENCRYPT {
            
            output[..buffer.len()].copy_from_slice(buffer);
            (output.as_ptr(), output.as_mut_ptr())
        } else {
            (buffer.as_ptr(), output.as_mut_ptr())
        };

        let ret = hhg_mbedtls_aes_crypt_cbc(
            handler, 
            mode as i32, 
            padded_len,  
            iv_copy.as_mut_ptr(), 
            input_ptr, 
            output_ptr
        );
        
        if ret != 0 {
            return Err(Error::ReturnWithCode(ret));
        }
    };

    Ok(output)
}

fn init() -> Result<*mut c_void> {
    unsafe { 
        let ret = hhg_mbedtls_aes_init();
        if ret.is_null() {
            Err(Error::NullPtr)
        } else {
            Ok(ret)
        }
    }
}

fn aes_encrypt(handler: *mut c_void, key: &[u8], iv: &[u8], plain: &[u8]) -> Result<Vec<u8>> {
    enc_dec(handler, aes_mode::AES_ENCRYPT, key, iv, plain)
}

fn aes_decrypt(handler: *mut c_void, key: &[u8], iv: &[u8], cipher: &[u8]) -> Result<Vec<u8>> {
    enc_dec(handler, aes_mode::AES_DECRYPT, key, iv, cipher)
}

#[allow(unused_assignments)]
pub fn get_sha256(data: &[u8]) -> Result<Bytes<{SHA256_RESULT_BYTES * 2}>> {
    let mut hash = Bytes::<SHA256_RESULT_BYTES>::new();

    let mut state: *mut c_void = null_mut();

    let ret = unsafe { hhg_pico_sha256_start_blocking(&mut state, true) };
    if ret != 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    if state.is_null() {
        return Err(Error::NullPtr);
    }

    unsafe {
        hhg_pico_sha256_update_blocking(state, data.as_ptr(), data.len());

        hhg_pico_sha256_finish(state, hash.as_mut_ptr());
    };

    state = null_mut(); // state is freed by finish, avoid dangling pointer

    let mut ret = Bytes::<{SHA256_RESULT_BYTES * 2}>::new();

    if bytes_to_hex_into_slice(hash.deref(), ret.deref_mut()) != SHA256_RESULT_BYTES * 2 {
        return Err(Error::Unhandled("Failed to convert hash to hex string"));
    }

    Ok(ret)
}

fn drop(handler: *mut c_void) {
    if handler.is_null() {
        return;
    }

    unsafe {
        hhg_mbedtls_aes_free(handler);
    }
}

