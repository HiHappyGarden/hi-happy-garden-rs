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
use osal_rs::log_info;
use osal_rs::utils::{Bytes, Error, Result};

use core::ffi::c_int;
use core::str::from_utf8;
pub use core::ffi::c_long as Handler;

use crate::drivers::pico::flash::{FILESYSTEM_FN, FILE_FN, DIR_FN};
use crate::traits::state::Initializable;

const APP_TAG: &str = "Filesystem";
const MAX_NAME_LEN: usize = 256;

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

/// Directory entry information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: Bytes<MAX_NAME_LEN>,
    pub type_: EntryType,
    pub size: u32,
}

/// Filesystem statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FsStat {
    pub block_size: u32,
    pub block_count: u32,
    pub blocks_used: u32,
}

/// File statistics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStat {
    pub name: Bytes<MAX_NAME_LEN>,
    pub type_: EntryType,
    pub size: u32,

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
    pub write: fn (handler: Handler, buffer: &[u8]) -> Result<isize>,

    /// Read data from the file
    pub read: fn (handler: Handler, buffer: &mut [u8]) -> Result<isize>, 

    /// Rewind file position to the beginning
    pub rewind: fn (handler: Handler) -> Result<()>,

    /// Seek to a position in the file
    pub seek: fn (handler: Handler, offset: i32, whence: i32) -> Result<isize>,

    /// Get current position in the file
    pub tell: fn (handler: Handler) -> Result<isize>,

    /// Truncate file to specified size
    pub truncate: fn (handler: Handler, size: u32) -> Result<()>,

    /// Flush file buffers
    pub flush: fn (handler: Handler) -> Result<()>,

    /// Get file size
    pub size: fn (handler: Handler) -> Result<isize>,

    //Close file
    pub close: fn (handler: Handler) -> Result<()>,
}

#[derive(Clone, Debug)]
pub struct DirFn {
    /// Read next entry in the directory
    pub read: fn (handler: Handler, type_: &mut u8, size: &mut u32, name: &mut [u8]) -> c_int,

    /// Seek to a position in the directory
    pub seek: fn (handler: Handler, offset: u32) -> Result<()>,

    /// Get current position in the directory
    pub tell: fn (handler: Handler) -> Result<i32>,
    
    /// Rewind directory position to the beginning
    pub rewind: fn (handler: Handler) -> Result<()>,

    /// Close Dir
    pub close: fn (handler: Handler) -> Result<()>,
}


#[derive(Clone, Debug)]
pub struct FilesystemFn {
    
    /// Mount the filesystem
    pub mount: fn (format: bool) -> Result<()>,

    /// Unmount the filesystem
    pub umount: fn () -> Result<()>,

    /// Open a file
    pub open: fn (path: &str, flags: i32) -> Result<isize>,

    /// Remove a file or directory
    pub remove: fn (path: &str) -> Result<()>,

    /// Rename a file or directory
    pub rename: fn (oldpath: &str, newpath: &str) -> Result<()>,

    /// Get filesystem statistics
    pub stat_fs: fn (block_size: &mut u32, block_count: &mut u32, blocks_used: &mut u32) -> Result<()>,

    /// Get file statistics
    pub stat: fn (path: &str, type_: &mut u8, size: &mut u32, name: &mut [u8]) -> Result<i32>,

    /// Get extended attribute
    pub getattr: fn (path: &str, type_: u8, buffer: &mut [u8]) -> Result<i32>,

    /// Set extended attribute
    pub setattr: fn (path: &str, type_: u8, buffer: &[u8]) -> Result<()>,

    /// Remove extended attribute
    pub removeattr: fn (path: &str, type_: u8) -> Result<()>,

    /// Create a directory
    pub mkdir: fn (path: &str) -> Result<()>,

    /// Open a directory
    pub open_dir: fn (path: &str) -> Result<isize>,

    /// Get error message for error code
    pub errmsg: fn (err: i32) -> &'static str
}

/// File handle wrapper
#[derive(Clone, Debug)]
pub struct File {
    functions: &'static FileFn,
    handler: Handler,
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = (self.functions.close)(self.handler);
    }
}

impl File {
    /// Write data to the file
    pub fn write(&self, buffer: &[u8]) -> Result<isize> {
        (self.functions.write)(self.handler, buffer)
    }

    /// Read data from the file
    pub fn read(&self, buffer: &mut [u8]) -> Result<isize> {
        (self.functions.read)(self.handler, buffer)
    }

    /// Rewind file position to the beginning
    pub fn rewind(&self) -> Result<()> {
        (self.functions.rewind)(self.handler)
    }

    /// Seek to a position in the file
    pub fn seek(&self, offset: i32, whence: SeekFrom) -> Result<isize> {
        (self.functions.seek)(self.handler, offset, whence.to_int())
    }

    /// Get current position in the file
    pub fn tell(&self) -> Result<isize> {
        (self.functions.tell)(self.handler)
    }

    /// Truncate file to specified size
    pub fn truncate(&self, size: u32) -> Result<()> {
        (self.functions.truncate)(self.handler, size)
    }

    /// Flush file buffers
    pub fn flush(&self) -> Result<()> {
        (self.functions.flush)(self.handler)
    }

    /// Get file size
    pub fn size(&self) -> Result<isize> {
        (self.functions.size)(self.handler)
    }
}

// /// Directory handle wrapper
#[derive(Clone, Debug)]
pub struct Dir {
    functions: &'static DirFn,  
    handler: Handler,
}

impl Dir {
    pub fn read(&self) -> Result<Option<DirEntry>> {
        let mut type_ = 0u8;
        let mut size = 0u32;
        let mut name = Bytes::<MAX_NAME_LEN>::new();


        let ret = (self.functions.read)(self.handler, &mut type_, &mut size, name.as_mut_slice());

        if ret < 0 {
            return Err(Error::ReturnWithCode(ret));
        }

        Ok(Some(DirEntry {
            name,
            type_: match EntryType::from_u8(type_) {
                EntryType::File => EntryType::File,
                EntryType::Dir => EntryType::Dir,
                EntryType::Unknown => EntryType::Unknown,
            },
            size,
        }))
    }

    pub fn seek(&self, offset: u32) -> Result<()> {
        (self.functions.seek)(self.handler, offset)
    }

    pub fn tell(&self) -> Result<i32> {
        (self.functions.tell)(self.handler)
    }

    pub fn rewind(&self) -> Result<()> {
        (self.functions.rewind)(self.handler)
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = (self.functions.close)(self.handler);
    }
}

/// Filesystem wrapper that implements FilesystemFn trait
pub struct Filesystem;


impl Filesystem {
    
    pub fn mount(format: bool) -> Result<()> {
        if (FILESYSTEM_FN.mount)(format).is_ok() {
            log_info!(APP_TAG, "Filesystem mounted");
            Ok(())
        } else {
            log_info!(APP_TAG, "Filesystem mount failed");
            Err(Error::Unhandled("Filesystem mount failed"))
        }
    }

    pub fn umount() -> Result<()> {
        (FILESYSTEM_FN.umount)()
    }

    pub fn open(path: &str, flags: i32) -> Result<File> {
        let handler = (FILESYSTEM_FN.open)(path, flags)?;
        Ok(File {
            functions: &FILE_FN,
            handler: handler as Handler,
        })
    }

    pub fn remove(path: &str) -> Result<()> {
        (FILESYSTEM_FN.remove)(path)
    }

    pub fn rename(oldpath: &str, newpath: &str) -> Result<()> {
        (FILESYSTEM_FN.rename)(oldpath, newpath)
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

    pub fn stat(path: &str) -> Result<FileStat> {
        let mut type_ = 0u8;
        let mut size = 0u32;
        let mut name = Bytes::<MAX_NAME_LEN>::new();

        let ret = (FILESYSTEM_FN.stat)(path, &mut type_, &mut size, name.as_mut_slice())?;

        if ret < 0 {
            return Err(Error::ReturnWithCode(ret));
        }

        Ok(FileStat {
            name,
            size,
            type_: EntryType::from_u8(type_),
        })
    }

    pub fn getattr(path: &str, type_: u8, buffer: &mut [u8]) -> Result<i32> {
        (FILESYSTEM_FN.getattr)(path, type_, buffer)
    }

    pub fn setattr(path: &str, type_: u8, buffer: &[u8]) -> Result<()> {
        (FILESYSTEM_FN.setattr)(path, type_, buffer)
    }

    pub fn removeattr(path: &str, type_: u8) -> Result<()> {
        (FILESYSTEM_FN.removeattr)(path, type_)
    }

    pub fn mkdir(path: &str) -> Result<()> {
        (FILESYSTEM_FN.mkdir)(path)
    }

    pub fn open_dir(path: &str) -> Result<Dir> {
        let handler = (FILESYSTEM_FN.open_dir)(path)?;
        Ok(Dir {
            functions: &DIR_FN,
            handler: handler as Handler,
        })
    }

    pub fn errmsg(err: i32) -> &'static str {
        (FILESYSTEM_FN.errmsg)(err)
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



//     unsafe { 
//         let aes = &mut *AES.get();
//         aes.0 = hhg_mbedtls_aes_init();
//         if aes.0.is_null() {
//             return Err(Error::NullPtr);
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

