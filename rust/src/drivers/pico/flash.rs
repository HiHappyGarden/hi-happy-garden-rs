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

use core::ffi::{CStr, c_int};
use core::str::from_utf8;

use alloc::ffi::CString;
use alloc::string::String;
use osal_rs::utils::{Error, Result};

use crate::drivers::pico::ffi::{
    LfsOff, LfsSize, LfsSoff, LfsSsize, hhg_flash_close, hhg_flash_dir_close, hhg_flash_dir_open, hhg_flash_dir_read, hhg_flash_dir_rewind, hhg_flash_dir_seek, hhg_flash_dir_tell, hhg_flash_errmsg, hhg_flash_fflush, hhg_flash_fsstat, hhg_flash_getattr, hhg_flash_lseek, hhg_flash_mkdir, hhg_flash_mount, hhg_flash_open, hhg_flash_read, hhg_flash_remove, hhg_flash_removeattr, hhg_flash_rename, hhg_flash_rewind, hhg_flash_setattr, hhg_flash_size, hhg_flash_stat, hhg_flash_tell, hhg_flash_truncate, hhg_flash_umount, hhg_flash_write
};
use crate::drivers::filesystem::{DirEntry, DirFn, EntryType, FileFn, FileStat, FilesystemFn, FsStat, Handler, SeekFrom}; 

const APP_TAG: &str = "Flash";
pub const FS_CONFIG_DIR: &str = "/etc";
pub const FS_DATA_DIR: &str = "/var";
pub const FS_LOG_DIR: &str = "/var/log";

pub const FILE_FN: FileFn = FileFn {
    
    open: file_open,

    write: file_write,

    read: file_read,

    rewind: file_rewind,

    seek: file_seek,

    tell: file_tell,

    truncate: file_truncate,

    flush: file_flush,

    size: file_size,

    close: file_close,
};




fn file_open(path: &str, flags: i32) -> Result<()> {
    let c_path = CString::new(path).map_err(|_| Error::InvalidType)?;
    let file = unsafe { hhg_flash_open(c_path.as_ptr(), flags) };
    if file < 0 {
        return Err(Error::ReturnWithCode(file as i32));
    }
    Ok(())
}

fn file_write(handler: Handler, buffer: &[u8]) -> Result<isize> {
    let written = unsafe {
        hhg_flash_write(
            handler,
            buffer.as_ptr() as *const _,
            buffer.len() as LfsSize,
        )
    };
    Ok(written as isize)
}

fn file_read(handler: Handler, buffer: &mut [u8]) -> Result<isize> {
    let read = unsafe {
        hhg_flash_read(
            handler,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as LfsSize,
        )
    };
    Ok(read as isize)
}

fn file_rewind(handler: Handler) -> Result<()> {
    let ret = unsafe { hhg_flash_rewind(handler) };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}

fn file_seek(handler: Handler, offset: i32, whence: i32) -> Result<isize> {
    let pos = unsafe { hhg_flash_lseek(handler, offset, whence) };
    if pos < 0 {
        return Err(Error::ReturnWithCode(pos));
    }
    Ok(pos as isize)
}


fn file_tell(handler: Handler) -> Result<isize> {
    let pos = unsafe { hhg_flash_tell(handler) };
    if pos < 0 {
        return Err(Error::ReturnWithCode(pos));
    }
    Ok(pos as isize)
}

fn file_truncate(handler: Handler, size: u32) -> Result<()> {
    let ret = unsafe { hhg_flash_truncate(handler, size) };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}

fn file_flush(handler: Handler) -> Result<()> {
    let ret = unsafe { hhg_flash_fflush(handler) };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}

fn file_size(handler: Handler) -> Result<isize> {
    let size = unsafe { hhg_flash_size(handler) };
    if size < 0 {
        return Err(Error::ReturnWithCode(size));
    }
    Ok(size as isize)
}

fn file_close(handler: Handler) -> Result<()> {
    let ret = unsafe { hhg_flash_close(handler) };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}

pub const DIR_FN: DirFn = DirFn {
    read: dir_read,
    seek: dir_seek,
    tell: dir_tell,
    rewind: dir_rewind,
    close: dir_close,
};

fn dir_read(handler: Handler, type_: &mut u8, size: &mut u32, name: &mut [u8]) -> c_int {
    unsafe {
        hhg_flash_dir_read(
            handler,
            type_,
            size,
            name.as_mut_ptr() as *mut _,
        )
    }
}

fn dir_seek(handler: Handler, offset: u32) -> Result<()> {
    let ret = unsafe { hhg_flash_dir_seek(handler, offset) };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}

fn dir_tell(handler: Handler) -> Result<i32> {
    let pos = unsafe { hhg_flash_dir_tell(handler) };
    if pos < 0 {
        return Err(Error::ReturnWithCode(pos));
    }
    Ok(pos as i32)
}

fn dir_rewind(handler: Handler) -> Result<()> {
    let ret = unsafe { hhg_flash_dir_rewind(handler) };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}   

fn dir_close(handler: Handler) -> Result<()> {
    let ret = unsafe { hhg_flash_dir_close(handler) };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}


pub const FILESYSTEM_FN: FilesystemFn = FilesystemFn {
    mount: filesystem_mount,

    umount: filesystem_umount,

    open: filesystem_open,

    remove: filesystem_remove,

    rename: filesystem_rename,

    stat_fs: filesystem_stat_fs,

    stat: filesystem_stat,

    getattr: filesystem_getattr,

    setattr: filesystem_setattr,

    removeattr: filesystem_removeattr,

    mkdir: filesystem_mkdir,

    open_dir: filesystem_open_dir,

    errmsg: filesystem_errmsg,
};

fn filesystem_mount(format: bool) -> Result<()> {
    let ret = unsafe { hhg_flash_mount(format) };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }

    


    Ok(())
}

fn filesystem_umount() -> Result<()> {
    let ret = unsafe { hhg_flash_umount() };
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}

fn filesystem_open(path: &str, flags: i32) -> Result<isize> {
    let c_path = CString::new(path).map_err(|_| Error::InvalidType)?;
    let handle = unsafe { hhg_flash_open(c_path.as_ptr(), flags) };
    if handle < 0 {
        return Err(Error::ReturnWithCode(handle as i32));
    }
    Ok(handle as isize)
}

fn filesystem_remove(path: &str) -> Result<()> {
    let c_path = CString::new(path).map_err(|_| Error::InvalidType)?;
    let ret = unsafe { hhg_flash_remove(c_path.as_ptr()) };
    
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}

fn filesystem_rename(oldpath: &str, newpath: &str) -> Result<()> {
    let oldpath_cstr = CString::new(oldpath).map_err(|_| Error::InvalidType)?;
    let newpath_cstr = CString::new(newpath).map_err(|_| Error::InvalidType)?;
    
    let ret = unsafe { hhg_flash_rename(oldpath_cstr.as_ptr(), newpath_cstr.as_ptr()) };
    
    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }
    Ok(())
}

fn filesystem_stat_fs(block_size: &mut u32, block_count: &mut u32, blocks_used: &mut u32) -> Result<()> {

    let ret = unsafe {
        hhg_flash_fsstat(block_size, block_count, blocks_used)
    };

    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }

    Ok(())
}

fn filesystem_stat(path: &str, type_: &mut u8, size: &mut u32, name: &mut [u8]) -> Result<i32> {
    let path_cstr = CString::new(path).map_err(|_| Error::InvalidType)?;

    let ret = unsafe {
        hhg_flash_stat(
            path_cstr.as_ptr(),
            type_,
            size,
            name.as_mut_ptr() as *mut _,
        )
    };

    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }

    Ok(ret)
}

fn filesystem_getattr(path: &str, type_: u8, buffer: &mut [u8]) -> Result<i32> {
    let path_cstr = CString::new(path).map_err(|_| Error::InvalidType)?;
    
    let ret = unsafe {
        hhg_flash_getattr(
            path_cstr.as_ptr(),
            type_,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as LfsSize,
        )
    };

    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }

    Ok(ret)
}

fn filesystem_setattr(path: &str, type_: u8, buffer: &[u8]) -> Result<()> {
    let path_cstr = CString::new(path).map_err(|_| Error::InvalidType)?;
    
    let ret = unsafe {
        hhg_flash_setattr(
            path_cstr.as_ptr(),
            type_,
            buffer.as_ptr() as *const _,
            buffer.len() as LfsSize,
        )
    };

    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }

    Ok(())
}

fn filesystem_removeattr(path: &str, type_: u8) -> Result<()> {
    let path_cstr = CString::new(path).map_err(|_| Error::InvalidType)?;
    
    let ret = unsafe { hhg_flash_removeattr(path_cstr.as_ptr(), type_) };

    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }

    Ok(())
}

fn filesystem_mkdir(path: &str) -> Result<()> {
    let path_cstr = CString::new(path).map_err(|_| Error::InvalidType)?;
    
    let ret = unsafe { hhg_flash_mkdir(path_cstr.as_ptr()) };

    if ret < 0 {
        return Err(Error::ReturnWithCode(ret));
    }

    Ok(())
}

fn filesystem_open_dir(path: &str) -> Result<isize> {
    let path_cstr = CString::new(path).map_err(|_| Error::InvalidType)?;
    
    let handle = unsafe { hhg_flash_dir_open(path_cstr.as_ptr()) };

    if handle < 0 {
        return Err(Error::ReturnWithCode(handle as i32));
    }

    Ok(handle as isize)
}

fn filesystem_errmsg(err: i32) -> &'static str {
    unsafe {
        let msg_ptr = hhg_flash_errmsg(err);
        if msg_ptr.is_null() {
            return "Unknown error";
        }
        
        let c_str = CStr::from_ptr(msg_ptr);
        c_str.to_str().unwrap_or("Invalid UTF-8 in error message")
    }
}