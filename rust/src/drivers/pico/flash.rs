#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_long, c_void};
use crate::drivers::pico::ffi::{
    hhg_flash_mount,
    hhg_flash_open,
    hhg_flash_close,
    hhg_flash_write,
    hhg_flash_read,
    hhg_flash_rewind,
    hhg_flash_umount,
    hhg_flash_remove,
    hhg_flash_rename,
    hhg_flash_fsstat,
    hhg_flash_lseek,
    hhg_flash_truncate,
    hhg_flash_tell,
    hhg_flash_stat,
    hhg_flash_getattr,
    hhg_flash_setattr,
    hhg_flash_removeattr,
    hhg_flash_fflush,
    hhg_flash_size,
    hhg_flash_mkdir,
    hhg_flash_dir_open,
    hhg_flash_dir_close,
    hhg_flash_dir_read,
    hhg_flash_dir_seek,
    hhg_flash_dir_tell,
    hhg_flash_dir_rewind,
    hhg_flash_errmsg,
    LfsSize,
    LfsSoff,
    LfsOff,
    LfsSsize,
};

/// Error type for flash operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashError {
    IoError(i32),
    InvalidHandle,
    InvalidPath,
    Utf8Error,
}

pub type Result<T> = core::result::Result<T, FlashError>;

/// File handle wrapper
#[derive(Debug)]
pub struct File {
    handle: c_long,
}

impl File {
    /// Write data to the file
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        let written = unsafe {
            hhg_flash_write(
                self.handle,
                buffer.as_ptr() as *const c_void,
                buffer.len() as LfsSize,
            )
        };
        Ok(written as usize)
    }

    /// Read data from the file
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let read = unsafe {
            hhg_flash_read(
                self.handle,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as LfsSize,
            )
        };
        Ok(read as usize)
    }

    /// Rewind file position to the beginning
    pub fn rewind(&mut self) -> Result<()> {
        let ret = unsafe { hhg_flash_rewind(self.handle) };
        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }
        Ok(())
    }

    /// Seek to a position in the file
    pub fn seek(&mut self, offset: i32, whence: SeekFrom) -> Result<i32> {
        let whence_val = whence.to_c_int();
        let pos = unsafe { hhg_flash_lseek(self.handle as c_int, offset, whence_val) };
        if pos < 0 {
            return Err(FlashError::IoError(pos));
        }
        Ok(pos)
    }

    /// Get current position in the file
    pub fn tell(&self) -> Result<i32> {
        let pos = unsafe { hhg_flash_tell(self.handle as c_int) };
        if pos < 0 {
            return Err(FlashError::IoError(pos));
        }
        Ok(pos)
    }

    /// Truncate file to specified size
    pub fn truncate(&mut self, size: u32) -> Result<()> {
        let ret = unsafe { hhg_flash_truncate(self.handle as c_int, size) };
        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }
        Ok(())
    }

    /// Flush file buffers
    pub fn flush(&mut self) -> Result<()> {
        let ret = unsafe { hhg_flash_fflush(self.handle as c_int) };
        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }
        Ok(())
    }

    /// Get file size
    pub fn size(&self) -> Result<i32> {
        let size = unsafe { hhg_flash_size(self.handle as c_int) };
        if size < 0 {
            return Err(FlashError::IoError(size));
        }
        Ok(size)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            hhg_flash_close(self.handle);
        }
    }
}

/// Directory handle wrapper
#[derive(Debug)]
pub struct Dir {
    handle: c_long,
}

impl Dir {
    /// Read next directory entry
    pub fn read(&mut self) -> Result<Option<DirEntry>> {
        let mut type_ = 0u8;
        let mut size = 0u32;
        let mut name_buf = [0u8; 256];

        let ret = unsafe {
            hhg_flash_dir_read(
                self.handle,
                &mut type_,
                &mut size,
                name_buf.as_mut_ptr() as *mut c_char,
            )
        };

        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }

        if ret == 0 {
            return Ok(None);
        }

        // Find null terminator
        let name_len = name_buf.iter().position(|&c| c == 0).unwrap_or(256);
        let name = alloc::string::String::from(
            core::str::from_utf8(&name_buf[..name_len])
                .map_err(|_| FlashError::Utf8Error)?
        );

        Ok(Some(DirEntry {
            name,
            type_: EntryType::from_u8(type_),
            size,
        }))
    }

    /// Seek to a position in the directory
    pub fn seek(&mut self, offset: u32) -> Result<()> {
        let ret = unsafe { hhg_flash_dir_seek(self.handle, offset) };
        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }
        Ok(())
    }

    /// Get current position in the directory
    pub fn tell(&self) -> Result<i32> {
        let pos = unsafe { hhg_flash_dir_tell(self.handle) };
        if pos < 0 {
            return Err(FlashError::IoError(pos));
        }
        Ok(pos)
    }

    /// Rewind directory to the beginning
    pub fn rewind(&mut self) -> Result<()> {
        let ret = unsafe { hhg_flash_dir_rewind(self.handle) };
        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }
        Ok(())
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        unsafe {
            hhg_flash_dir_close(self.handle);
        }
    }
}

/// Directory entry information
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: alloc::string::String,
    pub type_: EntryType,
    pub size: u32,
}

/// File/directory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Dir,
    Unknown,
}

impl EntryType {
    fn from_u8(val: u8) -> Self {
        match val {
            1 => EntryType::File,
            2 => EntryType::Dir,
            _ => EntryType::Unknown,
        }
    }
}

/// Seek position
#[derive(Debug, Clone, Copy)]
pub enum SeekFrom {
    Start(i32),
    Current(i32),
    End(i32),
}

impl SeekFrom {
    fn to_c_int(&self) -> c_int {
        match self {
            SeekFrom::Start(_) => 0,  // SEEK_SET
            SeekFrom::Current(_) => 1, // SEEK_CUR
            SeekFrom::End(_) => 2,    // SEEK_END
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

/// Filesystem statistics
#[derive(Debug, Clone, Copy)]
pub struct FsStat {
    pub block_size: u32,
    pub block_count: u32,
    pub blocks_used: u32,
}

/// File statistics
#[derive(Debug, Clone)]
pub struct FileStat {
    pub type_: EntryType,
    pub size: u32,
    pub name: alloc::string::String,
}

/// Flash filesystem API
pub struct Flash;

impl Flash {
    /// Mount the filesystem
    pub fn mount(format: bool) -> Result<()> {
        let ret = unsafe { hhg_flash_mount(format) };
        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }
        Ok(())
    }

    /// Unmount the filesystem
    pub fn umount() -> Result<()> {
        let ret = unsafe { hhg_flash_umount() };
        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }
        Ok(())
    }

    /// Open a file
    pub fn open(path: &str, flags: i32) -> Result<File> {
        let path_cstr = alloc::ffi::CString::new(path).map_err(|_| FlashError::InvalidPath)?;
        let handle = unsafe { hhg_flash_open(path_cstr.as_ptr(), flags) };
        
        if handle < 0 {
            return Err(FlashError::IoError(handle as i32));
        }

        Ok(File { handle })
    }

    /// Remove a file or directory
    pub fn remove(path: &str) -> Result<()> {
        let path_cstr = alloc::ffi::CString::new(path).map_err(|_| FlashError::InvalidPath)?;
        let ret = unsafe { hhg_flash_remove(path_cstr.as_ptr()) };
        
        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }
        Ok(())
    }

    /// Rename a file or directory
    pub fn rename(oldpath: &str, newpath: &str) -> Result<()> {
        let oldpath_cstr = alloc::ffi::CString::new(oldpath).map_err(|_| FlashError::InvalidPath)?;
        let newpath_cstr = alloc::ffi::CString::new(newpath).map_err(|_| FlashError::InvalidPath)?;
        
        let ret = unsafe { hhg_flash_rename(oldpath_cstr.as_ptr(), newpath_cstr.as_ptr()) };
        
        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }
        Ok(())
    }

    /// Get filesystem statistics
    pub fn stat_fs() -> Result<FsStat> {
        let mut block_size = 0u32;
        let mut block_count = 0u32;
        let mut blocks_used = 0u32;

        let ret = unsafe {
            hhg_flash_fsstat(&mut block_size, &mut block_count, &mut blocks_used)
        };

        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }

        Ok(FsStat {
            block_size,
            block_count,
            blocks_used,
        })
    }

    /// Get file statistics
    pub fn stat(path: &str) -> Result<FileStat> {
        let path_cstr = alloc::ffi::CString::new(path).map_err(|_| FlashError::InvalidPath)?;
        let mut type_ = 0u8;
        let mut size = 0u32;
        let mut name_buf = [0u8; 256];

        let ret = unsafe {
            hhg_flash_stat(
                path_cstr.as_ptr(),
                &mut type_,
                &mut size,
                name_buf.as_mut_ptr() as *mut c_char,
            )
        };

        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }

        let name_len = name_buf.iter().position(|&c| c == 0).unwrap_or(256);
        let name = alloc::string::String::from(
            core::str::from_utf8(&name_buf[..name_len])
                .map_err(|_| FlashError::Utf8Error)?
        );

        Ok(FileStat {
            type_: EntryType::from_u8(type_),
            size,
            name,
        })
    }

    /// Get extended attribute
    pub fn getattr(path: &str, type_: u8, buffer: &mut [u8]) -> Result<i32> {
        let path_cstr = alloc::ffi::CString::new(path).map_err(|_| FlashError::InvalidPath)?;
        
        let ret = unsafe {
            hhg_flash_getattr(
                path_cstr.as_ptr(),
                type_,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as LfsSize,
            )
        };

        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }

        Ok(ret)
    }

    /// Set extended attribute
    pub fn setattr(path: &str, type_: u8, buffer: &[u8]) -> Result<()> {
        let path_cstr = alloc::ffi::CString::new(path).map_err(|_| FlashError::InvalidPath)?;
        
        let ret = unsafe {
            hhg_flash_setattr(
                path_cstr.as_ptr(),
                type_,
                buffer.as_ptr() as *const c_void,
                buffer.len() as LfsSize,
            )
        };

        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }

        Ok(())
    }

    /// Remove extended attribute
    pub fn removeattr(path: &str, type_: u8) -> Result<()> {
        let path_cstr = alloc::ffi::CString::new(path).map_err(|_| FlashError::InvalidPath)?;
        
        let ret = unsafe { hhg_flash_removeattr(path_cstr.as_ptr(), type_) };

        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }

        Ok(())
    }

    /// Create a directory
    pub fn mkdir(path: &str) -> Result<()> {
        let path_cstr = alloc::ffi::CString::new(path).map_err(|_| FlashError::InvalidPath)?;
        
        let ret = unsafe { hhg_flash_mkdir(path_cstr.as_ptr()) };

        if ret < 0 {
            return Err(FlashError::IoError(ret));
        }

        Ok(())
    }

    /// Open a directory
    pub fn open_dir(path: &str) -> Result<Dir> {
        let path_cstr = alloc::ffi::CString::new(path).map_err(|_| FlashError::InvalidPath)?;
        let handle = unsafe { hhg_flash_dir_open(path_cstr.as_ptr()) };
        
        if handle < 0 {
            return Err(FlashError::IoError(handle as i32));
        }

        Ok(Dir { handle })
    }

    /// Get error message for error code
    pub fn errmsg(err: i32) -> &'static str {
        unsafe {
            let msg_ptr = hhg_flash_errmsg(err);
            if msg_ptr.is_null() {
                return "Unknown error";
            }
            
            let c_str = core::ffi::CStr::from_ptr(msg_ptr);
            c_str.to_str().unwrap_or("Invalid UTF-8 in error message")
        }
    }
}

// Required for String and CString
extern crate alloc;
