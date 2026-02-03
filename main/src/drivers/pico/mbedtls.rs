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

#![allow(unused)]

use core::ffi::c_void;

use alloc::vec::Vec;
use osal_rs::utils::{Error, Result};

use crate::drivers::pico::ffi::{aes_mode, hhg_mbedtls_aes_crypt_cbc, hhg_mbedtls_aes_free, hhg_mbedtls_aes_init, hhg_mbedtls_aes_setkey_enc};
use crate::drivers::ecnrypt::{EncryptFn}; 

const APP_TAG: &str = "MBEDTLS";

pub const ENCRYPT_FN: EncryptFn = EncryptFn {
    init,
    aes_encrypt,
    aes_decrypt,
    drop,
};

fn enc_dec(handler: *mut c_void, mode: aes_mode, key: &[u8], iv: &[u8], buffer: &[u8]) -> Result<Vec<u8>> {
    
    let keybits : u32 =  if key.len() == 16 {
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


    unsafe { 
        let ret = hhg_mbedtls_aes_setkey_enc(handler, key.as_ptr(), keybits);
        if ret != 0 {
            return Err(Error::ReturnWithCode(ret));
        }

        output[..buffer.len()].copy_from_slice(buffer);

        let iv = iv.to_vec();
        hhg_mbedtls_aes_crypt_cbc(handler, mode as i32, key.len() as usize, iv.as_ptr() as *mut _, buffer.as_ptr(), output.as_mut_ptr())
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

fn drop(handler: *mut c_void) {
    if handler.is_null() {
        return;
    }

    unsafe {
        hhg_mbedtls_aes_free(handler);
    }
}



// static KEY: SyncUnsafeCell<[u8; 16]> = SyncUnsafeCell::new([3u8; 16]);
// static AES: SyncUnsafeCell<MbedtlsAes> = SyncUnsafeCell::new(MbedtlsAes(null_mut()));


//     //TODO: enfore encryption initialization here
//     let mut id_buffer = [0u8; 8];
//     unsafe {
//         hhg_get_unique_id(id_buffer.as_mut_ptr());

//         let key = &mut *KEY.get();
//         for i in 0..id_buffer.len() * 2 {
//             key[i] = if i < id_buffer.len() {
//                 id_buffer[i]
//             } else {
//                 id_buffer[i - id_buffer.len()]
//             };
//         }
//     }






// fn enc_dec(mode: aes_mode, buffer: &[u8]) -> Result<Vec<u8>> {
    
//     let padded_len: usize = if mode == aes_mode::AES_ENCRYPT { 
//         (buffer.len() + 15) & !15usize
//     } else { 
//         buffer.len() + 1
//     };

//     let mut output: Vec<u8> = vec![0u8; padded_len];
    
//     unsafe { 
//         let aes = &*AES.get();
//         let key = &mut *KEY.get();
//         let ret = hhg_mbedtls_aes_setkey_enc(aes.0, key.as_ptr(), 128);
//         if ret != 0 {
//             return Err(Error::ReturnWithCode(ret));
//         }

//         output[..buffer.len()].copy_from_slice(buffer);

//         hhg_mbedtls_aes_crypt_cbc(aes.0, mode as i32, key.len() as usize, key.as_mut_ptr(), buffer.as_ptr(), output.as_mut_ptr())
//     };

//     Ok(output)
// }

