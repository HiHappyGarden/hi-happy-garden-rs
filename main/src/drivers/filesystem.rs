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

use alloc::ffi::CString;
use alloc::vec;
use alloc::vec::Vec;
use osal_rs::log_info;
use osal_rs::utils::{Bytes, Error, Result};

use core::ffi::c_int;
use core::str::from_utf8;
use core::ffi::c_void;
use core::ptr::null_mut;
use osal_rs::os::AsSyncStr;
use crate::drivers::encrypt::{Encrypt, EncryptGeneric};
use crate::drivers::pico::flash::{FILESYSTEM_FN, FILE_FN, DIR_FN};
use crate::drivers::platform::Hardware;
use crate::traits::state::Initializable;

const APP_TAG: &str = "Filesystem";
const MAX_NAME_LEN: usize = 256;

static mut ENCRYPT: Option<Encrypt<32, 16>> = None;
static mut KEY: [u8; 32] = [0u8; 32];
static mut IV: [u8; 16] = [0u8; 16];

/// Seek position enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    Start(i32),
    Current(i32),
    End(i32),
}

impl SeekFrom {
    pub fn to_int(&self) -> i32 {
        match self {
            SeekFrom::Start(_) => 0,  // SEEK_SET
            SeekFrom::Current(_) => 1, // SEEK_CUR
            SeekFrom::End(_) => 2,    // SEEK_END
        }
    }
}


/// Filesystem statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FsStat {
    pub block_size: u32,
    pub block_count: u32,
    pub blocks_used: u32,
}


/// File/directory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Dir,
    Unknown,
}

impl EntryType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => EntryType::File,
            2 => EntryType::Dir,
            _ => EntryType::Unknown,
        }
    }
}



/// File open flags
pub mod open_flags {
    pub const RDONLY: i32 = 0x0001;
    pub const WRONLY: i32 = 0x0002;
    pub const RDWR: i32 = 0x0003;
    pub const CREAT: i32 = 0x0100;
    pub const EXCL: i32 = 0x0200;
    pub const TRUNC: i32 = 0x0400;
    pub const APPEND: i32 = 0x0800;
}

#[derive(Clone, Debug)]
pub struct FileFn {
    //Open file
    pub open: fn (path: &str, flags: i32) -> Result<()>,

    /// Write data to the file
    pub write: fn (handler: *mut c_void, buffer: &[u8]) -> Result<isize>,

    /// Read data from the file
    pub read: fn (handler: *mut c_void) -> Result<Vec<u8>>, 

    /// Rewind file position to the beginning
    pub rewind: fn (handler: *mut c_void) -> Result<()>,

    /// Seek to a position in the file
    pub seek: fn (handler: *mut c_void, offset: i32, whence: i32) -> Result<isize>,

    /// Get current position in the file
    pub tell: fn (handler: *mut c_void) -> Result<isize>,

    /// Truncate file to specified size
    pub truncate: fn (handler: *mut c_void, size: u32) -> Result<()>,

    /// Flush file buffers
    pub flush: fn (handler: *mut c_void) -> Result<()>,

    /// Get file size
    pub size: fn (handler: *mut c_void) -> Result<isize>,

    ///Close file
    pub close: fn (handler: *mut c_void) -> Result<()>,
}

#[derive(Clone, Debug)]
pub struct DirFn {
    /// Read next entry in the directory
    pub read: fn (handler: *mut c_void, type_: &mut u8, size: &mut u32, name: &mut [u8]) -> c_int,

    /// Seek to a position in the directory
    pub seek: fn (handler: *mut c_void, offset: u32) -> Result<()>,

    /// Get current position in the directory
    pub tell: fn (handler: *mut c_void) -> Result<i32>,
    
    /// Rewind directory position to the beginning
    pub rewind: fn (handler: *mut c_void) -> Result<()>,

    /// Close Dir
    pub close: fn (handler: *mut c_void) -> Result<()>,
}


#[derive(Clone, Debug)]
pub struct FilesystemFn {
    
    /// Mount the filesystem
    pub mount: fn (format: bool) -> Result<()>,

    /// Unmount the filesystem
    pub umount: fn () -> Result<()>,

    /// Open a file
    pub open: fn (path: &str, flags: i32) -> Result<*mut c_void>,

    /// Remove a file or directory
    pub remove: fn (path: &str) -> Result<()>,

    /// Rename a file or directory
    pub rename: fn (old_path: &str, new_path: &str) -> Result<()>,

    /// Get filesystem statistics
    pub stat_fs: fn (block_size: &mut u32, block_count: &mut u32, blocks_used: &mut u32) -> Result<()>,

    /// Get file statistics
    pub stat: fn (path: &str, type_: &mut u8, size: &mut u32, name: &mut [u8]) -> Result<i32>,

    /// Get extended attribute
    pub get_attr: fn (path: &str, type_: u8, buffer: &mut [u8]) -> Result<i32>,

    /// Set extended attribute
    pub set_attr: fn (path: &str, type_: u8, buffer: &[u8]) -> Result<()>,

    /// Remove extended attribute
    pub remove_attr: fn (path: &str, type_: u8) -> Result<()>,

    /// Create a directory
    pub mkdir: fn (path: &str) -> Result<()>,

    /// Open a directory
    pub open_dir: fn (path: &str) -> Result<*mut c_void>,

    /// Get error message for error code
    pub err_msg: fn (err: i32) -> &'static str
}


/// File handle wrapper
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    handler: *mut c_void,
    pub name: Bytes<MAX_NAME_LEN>,
    pub size: u32,
}

impl Drop for File {
    fn drop(&mut self) {
        if self.handler.is_null() {
            return;
        }
        let _ = (FILE_FN.close)(self.handler);
    }
}

impl File {

    pub(super) fn new (name: Bytes::<MAX_NAME_LEN>, size: u32) -> Self {
        Self {
            handler: null_mut(),
            name,
            size,
        }
    }

    /// Write data to the file
    #[inline]
    pub fn write_with_as_sync_str(&mut self, buffer: &impl AsSyncStr) -> Result<isize> {
        self.write(buffer.as_str().as_bytes())
    }

    /// Write data to the file
    pub fn write(&mut self, buffer: &[u8]) -> Result<isize> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }

        #[cfg(feature = "encryption")]
        {
            let encrypted_buffer = unsafe { (*&raw const ENCRYPT).unwrap().aes_encrypt(buffer)? };

            let ret = (FILE_FN.write)(self.handler, &encrypted_buffer)?;

            self.size = ret as u32;

            let mut buffer_sha_file =  self.name.clone();
            buffer_sha_file.append_str(".sha256");

            let mut file_sha = Filesystem::open(buffer_sha_file.as_str(), open_flags::WRONLY | open_flags::CREAT)?;
            file_sha.write(EncryptGeneric::get_sha256(buffer)?.as_slice())?;
            file_sha.close()?;

            Ok(ret)
        }

        #[cfg(not(feature = "encryption"))]
        {
            let ret = (FILE_FN.write)(self.0, buffer)?;

            self.size = ret as u32;

            let mut buffer_sha_file =  self.name.clone();
            buffer_sha_file.append_str(".sha256");

            let mut file_sha = Filesystem::open(buffer_sha_file.as_str(), open_flags::WRONLY | open_flags::CREAT)?;
            file_sha.write(EncryptGeneric::get_sha256(buffer)?.as_slice())?;
            file_sha.close()?;

            Ok(ret)
        }

    }

    /// Read data from the file
    pub fn read(&mut self) -> Result<Vec<u8>> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }

        #[cfg(feature = "encryption")]
        {
            let encrypted_buffer = (FILE_FN.read)(self.handler)?;
            self.size = encrypted_buffer.len() as u32;

            let mut decrypted_buffer = unsafe { (*&raw const ENCRYPT).unwrap().aes_decrypt(&encrypted_buffer)? };

            while decrypted_buffer.last() == Some(&0) {
                decrypted_buffer.pop();
            }

            let mut buffer_sha_file =  self.name.clone();
            buffer_sha_file.append_str(".sha256");

            let mut file_sha = Filesystem::open(buffer_sha_file.as_str(), open_flags::RDONLY)?;
            let sha256_stored = file_sha.read()?;
            file_sha.close()?;

            let sha256_computed = EncryptGeneric::get_sha256(&decrypted_buffer)?;

            if sha256_stored != sha256_computed.as_slice() {
                return Err(Error::ReadError("Data integrity check failed. SHA256 hash does not match."));
            }

            Ok(decrypted_buffer)
        }

        #[cfg(not(feature = "encryption"))]
        {
            let buffer = (FILE_FN.read)(self.0)?;

            self.size = buffer.len() as u32;

            Ok(buffer)
        }

    }

    /// Rewind file position to the beginning
    pub fn rewind(&self) -> Result<()> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }
        (FILE_FN.rewind)(self.handler)
    }

    /// Seek to a position in the file
    pub fn seek(&self, offset: i32, whence: SeekFrom) -> Result<isize> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }
        (FILE_FN.seek)(self.handler, offset, whence.to_int())
    }

    /// Get current position in the file
    pub fn tell(&self) -> Result<isize> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }
        (FILE_FN.tell)(self.handler)
    }

    /// Truncate file to specified size
    pub fn truncate(&self, size: u32) -> Result<()> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }
        (FILE_FN.truncate)(self.handler, size)
    }

    /// Flush file buffers
    pub fn flush(&self) -> Result<()> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }
        (FILE_FN.flush)(self.handler)
    }

    /// Get file size
    pub fn size(&self) -> Result<isize> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }
        (FILE_FN.size)(self.handler)
    }

    pub fn close(&mut self) -> Result<()> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }
        (FILE_FN.close)(self.handler)?;
        self.handler = null_mut();
        Ok(())
    }
}

// /// Directory handle wrapper
#[derive(Clone, Debug)]
pub struct Dir {
    handler: *mut c_void,
    pub name: Bytes<MAX_NAME_LEN>,
    pub type_: EntryType,

}

impl Drop for Dir {
    fn drop(&mut self) {
        if self.handler.is_null() {
            return;
        }
        let _ = (DIR_FN.close)(self.handler);
    }
}

impl Dir {

    fn new (name: Bytes::<MAX_NAME_LEN>, type_: EntryType) -> Self {
        Self {
            handler: null_mut(),
            name,
            type_,
        }
    }

    pub fn read(&self) -> Result<Self> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }

        let mut type_ = 0u8;
        let mut size = 0u32;
        let mut name = Bytes::<MAX_NAME_LEN>::new();


        let ret = (DIR_FN.read)(self.handler, &mut type_, &mut size, name.as_mut_slice());

        if ret < 0 {
            return Err(Error::ReturnWithCode(ret));
        }

        Ok(Self::new(name, EntryType::from_u8(type_)))
    }

    pub fn seek(&self, offset: u32) -> Result<()> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }

        (DIR_FN.seek)(self.handler, offset)
    }

    pub fn tell(&self) -> Result<i32> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }

        (DIR_FN.tell)(self.handler)
    }

    pub fn rewind(&self) -> Result<()> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }

        (DIR_FN.rewind)(self.handler)
    }

    pub fn close(&mut self) -> Result<()> {
        if self.handler.is_null() {
            return Err(Error::NullPtr);
        }
        (DIR_FN.close)(self.handler)?;
        self.handler = null_mut();
        Ok(())
    }
}

/// Filesystem wrapper that implements FilesystemFn trait
pub struct Filesystem;


impl Filesystem {
    
    pub fn mount(format: bool) -> Result<()> {

        unsafe {

            //todo: Enforce unique key/iv generation only once
            let unique_id = Hardware::get_unique_id();

            for i in 0..(*&raw const KEY).len() as usize {
                KEY[i] = unique_id[i % unique_id.len()];
            }
    
            for i in 0..(*&raw const IV).len() as usize {
                IV[i] = unique_id[(i + 3) % unique_id.len()];
            }

            let mut encrypt = Encrypt::new(&*&raw const KEY, &*&raw const IV)?;
            encrypt.init()?;
            ENCRYPT = Some(encrypt);
        }

        if (FILESYSTEM_FN.mount)(format).is_ok() {
            log_info!(APP_TAG, "Filesystem mounted");
            Ok(())
        } else {
            log_info!(APP_TAG, "Filesystem mount failed");
            Err(Error::Unhandled("Filesystem mount failed"))
        }
    }

    pub fn umount() -> Result<()> {
        unsafe {
            let _ = &ENCRYPT.unwrap().drop();
        }
        (FILESYSTEM_FN.umount)()
    }

    #[inline]
    pub fn open_with_as_sync_str(path: &impl AsSyncStr, flags: i32) -> Result<File>  {
        Filesystem::open(path.as_str(), flags)
    }


    pub fn open(path: &str, flags: i32) -> Result<File> {
        let handler = (FILESYSTEM_FN.open)(path, flags)?;
        Ok(File {
            handler,
            name: Bytes::<MAX_NAME_LEN>::new_by_str(path),
            size: 0,
        })
    }

    #[inline]
    pub fn remove(path: &str) -> Result<()> {
        (FILESYSTEM_FN.remove)(path)
    }

    #[inline]
    pub fn remove_with_as_sync_str(path: &impl AsSyncStr) -> Result<()>  {
        Filesystem::remove(path.as_str())
    }

    #[inline]
    pub fn rename(old_path: &str, new_path: &str) -> Result<()> {
        (FILESYSTEM_FN.rename)(old_path, new_path)
    }

    #[inline]
    pub fn rename_with_as_sync_str(old_path: &impl AsSyncStr, new_path: &impl AsSyncStr) -> Result<()>  {
        Filesystem::rename(old_path.as_str(), new_path.as_str())
    }

    pub fn stat_fs() -> Result<FsStat> {
        let mut block_size = 0u32;
        let mut block_count = 0u32;
        let mut blocks_used = 0u32;

        (FILESYSTEM_FN.stat_fs)(&mut block_size, &mut block_count, &mut blocks_used)?;

        Ok(FsStat {
            block_size,
            block_count,
            blocks_used,
        })
    }

    pub fn stat(path: &str) -> Result<File> {
        let mut type_ = 0u8;
        let mut size = 0u32;
        let mut name = Bytes::<MAX_NAME_LEN>::new();

        let ret = (FILESYSTEM_FN.stat)(path, &mut type_, &mut size, name.as_mut_slice())?;

        if ret < 0 {
            return Err(Error::ReturnWithCode(ret));
        }

        Ok(File::new(name, size))
    }

    #[inline]
    pub fn stat_with_as_sync_str(path: &impl AsSyncStr) -> Result<File> {
        Filesystem::stat(path.as_str())
    }

    #[inline]
    pub fn get_attr(path: &str, type_: u8, buffer: &mut [u8]) -> Result<i32> {
        (FILESYSTEM_FN.get_attr)(path, type_, buffer)
    }

    #[inline]
    pub fn get_attr_with_as_sync_str(path: &impl AsSyncStr, type_: u8, buffer: &mut [u8]) -> Result<i32> {
        (FILESYSTEM_FN.get_attr)(path.as_str(), type_, buffer)
    }

    #[inline]
    pub fn set_attr(path: &str, type_: u8, buffer: &[u8]) -> Result<()> {
        (FILESYSTEM_FN.set_attr)(path, type_, buffer)
    }

    #[inline]
    pub fn set_attr_with_as_sync_str(path: &impl AsSyncStr, type_: u8, buffer: &[u8]) -> Result<()> {
        (FILESYSTEM_FN.set_attr)(path.as_str(), type_, buffer)
    }

    #[inline]
    pub fn remove_attr(path: &str, type_: u8) -> Result<()> {
        (FILESYSTEM_FN.remove_attr)(path, type_)
    }

    #[inline]
    pub fn remove_attr_with_as_sync_str(path: &impl AsSyncStr, type_: u8) -> Result<()> {
        (FILESYSTEM_FN.remove_attr)(path.as_str(), type_)
    }

    #[inline]
    pub fn mkdir(path: &str) -> Result<()> {
        (FILESYSTEM_FN.mkdir)(path)
    }

    #[inline]
    pub fn mkdir_attr_with_as_sync_str(path: &impl AsSyncStr) -> Result<()> {
        (FILESYSTEM_FN.mkdir)(path.as_str())
    }

    pub fn open_dir(path: &str) -> Result<Dir> {
        let handler = (FILESYSTEM_FN.open_dir)(path)?;
        Ok(Dir {
            handler,
            name: Bytes::<MAX_NAME_LEN>::new_by_str(path),
            type_: EntryType::Dir,
        })
    }

    #[inline]
    pub fn open_dir_with_as_sync_str(path: &impl AsSyncStr) -> Result<Dir> {
        Filesystem::open_dir(path.as_str())
    }

    pub fn err_msg(err: i32) -> &'static str {
        (FILESYSTEM_FN.err_msg)(err)
    }
}


