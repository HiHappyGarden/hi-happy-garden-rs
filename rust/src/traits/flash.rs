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

use alloc::string::String;
use osal_rs::utils::Result;

/// Seek position enum
#[derive(Debug, Clone, Copy)]
pub enum SeekFrom {
    Start(i32),
    Current(i32),
    End(i32),
}

/// File/directory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Dir,
    Unknown,
}

/// Directory entry information
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub type_: EntryType,
    pub size: u32,
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
    pub name: String,
}

/// Trait for file operations
pub trait FileFn {
    /// Write data to the file
    fn write(&mut self, buffer: &[u8]) -> Result<usize>;

    /// Read data from the file
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize>;

    /// Rewind file position to the beginning
    fn rewind(&mut self) -> Result<()>;

    /// Seek to a position in the file
    fn seek(&mut self, offset: i32, whence: SeekFrom) -> Result<i32>;

    /// Get current position in the file
    fn tell(&self) -> Result<i32>;

    /// Truncate file to specified size
    fn truncate(&mut self, size: u32) -> Result<()>;

    /// Flush file buffers
    fn flush(&mut self) -> Result<()>;

    /// Get file size
    fn size(&self) -> Result<i32>;
}

/// Trait for directory operations
pub trait DirFn {
    /// Read next directory entry
    fn read(&mut self) -> Result<Option<DirEntry>>;

    /// Seek to a position in the directory
    fn seek(&mut self, offset: u32) -> Result<()>;

    /// Get current position in the directory
    fn tell(&self) -> Result<i32>;

    /// Rewind directory to the beginning
    fn rewind(&mut self) -> Result<()>;
}

/// Trait for filesystem operations
pub trait FilesystemFn {
    /// File handle type
    type File: FileFn;
    
    /// Directory handle type
    type Dir: DirFn;

    /// Open a file
    fn open(path: &str, flags: i32) -> Result<Self::File>;

    /// Remove a file or directory
    fn remove(path: &str) -> Result<()>;

    /// Rename a file or directory
    fn rename(oldpath: &str, newpath: &str) -> Result<()>;

    /// Get filesystem statistics
    fn stat_fs() -> Result<FsStat>;

    /// Get file statistics
    fn stat(path: &str) -> Result<FileStat>;

    /// Get extended attribute
    fn getattr(path: &str, type_: u8, buffer: &mut [u8]) -> Result<i32>;

    /// Set extended attribute
    fn setattr(path: &str, type_: u8, buffer: &[u8]) -> Result<()>;

    /// Remove extended attribute
    fn removeattr(path: &str, type_: u8) -> Result<()>;

    /// Create a directory
    fn mkdir(path: &str) -> Result<()>;

    /// Open a directory
    fn open_dir(path: &str) -> Result<Self::Dir>;

    /// Get error message for error code
    fn errmsg(err: i32) -> &'static str;
}

