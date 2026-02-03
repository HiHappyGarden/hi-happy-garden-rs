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

#![allow(dead_code)]

use core::{ffi::c_void, ptr::null_mut};

use alloc::{str, vec::Vec};
use osal_rs::{log_info, utils::{Error, Result}};

use crate::traits::state::Initializable;
use crate::drivers::pico::mbedtls::ENCRYPT_FN;

const APP_TAG: &str = "Encrypt";

pub struct EncryptFn {
    pub init: fn() -> Result<*mut c_void>,
    pub aes_encrypt: fn(handler: *mut c_void, key: &[u8], iv: &[u8], plain: &[u8]) -> Result<Vec<u8>>,
    pub aes_decrypt: fn(handler: *mut c_void, key: &[u8], iv: &[u8], cipher: &[u8]) -> Result<Vec<u8>>,
    pub drop: fn(*mut c_void),
}

#[derive(Clone, Copy)]
pub struct Encrypt<'a, const KEY_SIZE: usize = 16, const IV_SIZE: usize = 16> {
    functions: &'static EncryptFn,
    handler: *mut c_void,
    key: &'a [u8; KEY_SIZE], 
    iv: &'a [u8; IV_SIZE],
}

impl<'a, const KEY_SIZE: usize, const IV_SIZE: usize> Initializable for Encrypt<'a, KEY_SIZE, IV_SIZE> {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init encrypt");

        self.handler = (self.functions.init)()?;


        Ok(())
    }
}

impl<'a, const KEY_SIZE: usize, const IV_SIZE: usize> Encrypt<'a, KEY_SIZE, IV_SIZE> {
    pub const fn new(key: &'a [u8; KEY_SIZE], iv: &'a [u8; IV_SIZE]) -> Result<Self> {
        if KEY_SIZE != 16 && KEY_SIZE != 24 && KEY_SIZE != 32 {
            return Err(Error::Unhandled("Invalid key size. Must be 16, 24, or 32 bytes."));
        }

        if IV_SIZE != 16 {
            return Err(Error::Unhandled("Invalid IV size. Must be 16 bytes."));
        }

        Ok(Self {
            functions: &ENCRYPT_FN,
            handler: null_mut(),
            key,
            iv,
        })
    }

    pub fn aes_encrypt(&self, plain: &[u8]) -> Result<Vec<u8>> {
        (self.functions.aes_encrypt)(self.handler,  self.key, self.iv, plain)
    }

    pub fn aes_decrypt(&self, cipher: &[u8]) -> Result<Vec<u8>> {
        (self.functions.aes_decrypt)(self.handler, self.key, self.iv, cipher)
    }

    pub fn drop(&mut self) {
        log_info!(APP_TAG, "Free encrypt");

        (self.functions.drop)(self.handler);
    }
}
