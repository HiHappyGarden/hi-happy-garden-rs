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

//! Filesystem tests on the real littlefs flash partition, mounted by
//! `Hardware::init`.
//!
//! Everything happens under [`TEST_DIR`], which is wiped at the start (left
//! over by an aborted run) and removed at the end, so the device
//! configuration in `/etc` is never touched.

use alloc::vec::Vec;

use osal_rs::log_debug;
use osal_rs::utils::{Error, Result};

use super::flags::{CREAT, RDONLY, TRUNC, WRONLY};
use super::{EntryType, FileBytes, Filesystem, SeekFrom};
use crate::drivers::platform::{FS_CONFIG_DIR, FS_DATA_DIR, FS_LOG_DIR};
use crate::drivers::plt::flash::lfs_errors::{LFS_ERR_BADF, LFS_ERR_EXIST, LFS_ERR_NOENT};
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};

const TAG: &str = "FilesystemTests";

const TEST_DIR: &str = "/var/hhg_test";

fn path(name: &str) -> FileBytes {
    let mut path = FileBytes::from_str(TEST_DIR);
    path.append_str("/");
    path.append_str(name);
    path
}

fn write_file(name: &FileBytes, data: &[u8], sha256: bool) -> Result<isize> {
    let mut file = Filesystem::open_with_as_sync_str(name, WRONLY | CREAT | TRUNC)?;
    file.write(data, sha256)
}

fn read_file(name: &FileBytes, sha256: bool) -> Result<Vec<u8>> {
    let mut file = Filesystem::open_with_as_sync_str(name, RDONLY)?;
    file.read(sha256)
}

fn is_noent<T>(result: Result<T>) -> bool {
    matches!(result, Err(Error::ReturnWithCode(LFS_ERR_NOENT)))
}

fn setup() -> Result<()> {
    if Filesystem::stat(TEST_DIR).is_ok() {
        Filesystem::remove_recursive(TEST_DIR)?;
    }
    Filesystem::mkdir(TEST_DIR)
}

fn test_stat_fs() -> Result<()> {
    let stat = Filesystem::stat_fs()?;
    log_debug!(TAG, "{:?}", stat);
    test_assert!(stat.block_size > 0);
    test_assert!(stat.block_count > 0);
    test_assert!(stat.blocks_used <= stat.block_count);
    // Leave room for config saves
    test_assert!(stat.blocks_used < stat.block_count, "filesystem full");
    Ok(())
}

fn test_system_dirs() -> Result<()> {
    for dir in [FS_CONFIG_DIR, FS_DATA_DIR, FS_LOG_DIR] {
        test_assert!(Filesystem::stat(dir).is_ok(), "missing {dir}");
    }
    // mkdir on an existing directory must report EXIST, Hardware::init_fs relies on it
    test_assert!(matches!(Filesystem::mkdir(FS_CONFIG_DIR), Err(Error::ReturnWithCode(LFS_ERR_EXIST))));
    Ok(())
}

fn test_open_missing() -> Result<()> {
    test_assert!(is_noent(Filesystem::open_with_as_sync_str(&path("missing.txt"), RDONLY)));
    test_assert!(is_noent(Filesystem::stat_with_as_sync_str(&path("missing.txt"))));
    test_assert!(is_noent(Filesystem::remove_with_as_sync_str(&path("missing.txt"))));
    Ok(())
}

fn test_write_read_roundtrip() -> Result<()> {
    let name = path("roundtrip.bin");
    let data: Vec<u8> = (1..=200u8).collect();

    let written = write_file(&name, &data, false)?;
    // AES-CBC with zero padding: 200 bytes take 13 blocks on flash
    test_assert_eq!(written, 208);

    test_assert_eq!(read_file(&name, false)?, data);
    test_assert_eq!(Filesystem::stat_with_as_sync_str(&name)?.size, 208);
    Ok(())
}

fn test_data_is_encrypted() -> Result<()> {
    let name = path("plain.txt");
    let data = b"this text must not be on flash in clear";
    write_file(&name, data, false)?;

    // Read the raw bytes bypassing File::read, which decrypts
    let file = Filesystem::open_with_as_sync_str(&name, RDONLY)?;
    let raw = (super::FILE_FN.read)(file.handler)?;
    test_assert_eq!(raw.len(), 48);
    test_assert!(raw.windows(data.len()).all(|w| w != data), "plaintext found on flash");
    Ok(())
}

fn test_truncate_on_rewrite() -> Result<()> {
    let name = path("truncate.txt");
    write_file(&name, &[b'x'; 100], false)?;
    write_file(&name, b"short", false)?;
    test_assert_eq!(read_file(&name, false)?, b"short");
    Ok(())
}

fn test_empty_file() -> Result<()> {
    let name = path("empty.txt");
    drop(Filesystem::open_with_as_sync_str(&name, WRONLY | CREAT | TRUNC)?);
    test_assert!(read_file(&name, false)?.is_empty());
    test_assert_eq!(Filesystem::stat_with_as_sync_str(&name)?.size, 0);
    Ok(())
}

fn test_sha256_integrity() -> Result<()> {
    let name = path("signed.json");
    let data = br#"{"key":"value"}"#;
    write_file(&name, data, true)?;

    let mut sha_name = name.clone();
    sha_name.append_str(".sha256");
    test_assert!(Filesystem::stat_with_as_sync_str(&sha_name).is_ok(), "missing .sha256 companion");

    test_assert_eq!(read_file(&name, true)?, data);

    // Corrupt the stored hash: the read must refuse the data
    let mut sha_file = Filesystem::open_with_as_sync_str(&sha_name, WRONLY | TRUNC)?;
    sha_file.write(&[b'0'; 64], false)?;
    drop(sha_file);
    test_assert!(matches!(read_file(&name, true), Err(Error::ReadError(_))), "tampered hash accepted");

    // Without the integrity check the data is still readable
    test_assert_eq!(read_file(&name, false)?, data);
    Ok(())
}

fn test_seek_tell_size() -> Result<()> {
    let name = path("seek.bin");
    write_file(&name, &[0xAA; 32], false)?;

    let file = Filesystem::open_with_as_sync_str(&name, RDONLY)?;
    test_assert_eq!(file.size()?, 32);
    test_assert_eq!(file.tell()?, 0);
    test_assert_eq!(file.seek(16, SeekFrom::Start(16))?, 16);
    test_assert_eq!(file.tell()?, 16);
    test_assert_eq!(file.seek(0, SeekFrom::End(0))?, 32);
    file.rewind()?;
    test_assert_eq!(file.tell()?, 0);
    Ok(())
}

fn test_read_error() -> Result<()> {
    let name = path("write_only.txt");
    write_file(&name, b"not empty", false)?;

    // A failed lfs read (negative length) must come back as an error, not a panic
    let mut file = Filesystem::open_with_as_sync_str(&name, WRONLY)?;
    test_assert!(matches!((super::FILE_FN.read)(file.handler), Err(Error::ReturnWithCode(LFS_ERR_BADF))));
    test_assert!(matches!(file.read(false), Err(Error::ReturnWithCode(LFS_ERR_BADF))));
    Ok(())
}

fn test_write_error() -> Result<()> {
    let name = path("read_only.txt");
    write_file(&name, b"keep me", false)?;

    // A failed lfs write (negative length) must come back as an error, not as Ok
    {
        let mut file = Filesystem::open_with_as_sync_str(&name, RDONLY)?;
        test_assert!(matches!((super::FILE_FN.write)(file.handler, b"x"), Err(Error::ReturnWithCode(LFS_ERR_BADF))));
        test_assert!(matches!(file.write(b"overwritten", false), Err(Error::ReturnWithCode(LFS_ERR_BADF))));
    }
    test_assert_eq!(read_file(&name, false)?, b"keep me");
    Ok(())
}

fn test_closed_file() -> Result<()> {
    let name = path("closed.txt");
    let mut file = Filesystem::open_with_as_sync_str(&name, WRONLY | CREAT | TRUNC)?;
    file.close()?;
    test_assert!(matches!(file.write(b"x", false), Err(Error::NullPtr)));
    test_assert!(matches!(file.read(false), Err(Error::NullPtr)));
    test_assert!(matches!(file.close(), Err(Error::NullPtr)));

    // A File built by stat() has no handle at all
    let stat = Filesystem::stat_with_as_sync_str(&name)?;
    test_assert!(matches!(stat.size(), Err(Error::NullPtr)));
    Ok(())
}

fn test_rename_remove() -> Result<()> {
    let old = path("old.txt");
    let new = path("new.txt");
    write_file(&old, b"rename me", false)?;

    Filesystem::rename_with_as_sync_str(&old, &new)?;
    test_assert!(is_noent(Filesystem::stat_with_as_sync_str(&old)));
    test_assert_eq!(read_file(&new, false)?, b"rename me");

    Filesystem::remove_with_as_sync_str(&new)?;
    test_assert!(is_noent(Filesystem::stat_with_as_sync_str(&new)));
    Ok(())
}

fn test_ls() -> Result<()> {
    let mut dir = FileBytes::from_str(TEST_DIR);
    dir.append_str("/ls");
    Filesystem::mkdir_attr_with_as_sync_str(&dir)?;

    for name in ["a.txt", "b.txt"] {
        let mut file = dir.clone();
        file.append_str("/");
        file.append_str(name);
        write_file(&file, name.as_bytes(), false)?;
    }
    let mut sub = dir.clone();
    sub.append_str("/sub");
    Filesystem::mkdir_attr_with_as_sync_str(&sub)?;

    let entries = Filesystem::ls_with_as_sync_str(&dir)?;
    log_debug!(TAG, "ls {dir}: {:?}", entries);
    let has = |name: &str, type_: EntryType| entries.iter().any(|(n, t)| n == name && *t == type_);
    test_assert!(has("a.txt", EntryType::File));
    test_assert!(has("b.txt", EntryType::File));
    test_assert!(has("sub", EntryType::Dir));
    test_assert_eq!(entries.iter().filter(|(n, _)| n != "." && n != "..").count(), 3);
    Ok(())
}

fn test_remove_recursive() -> Result<()> {
    let mut nested = FileBytes::from_str(TEST_DIR);
    nested.append_str("/r");
    Filesystem::mkdir_attr_with_as_sync_str(&nested)?;
    nested.append_str("/deep");
    Filesystem::mkdir_attr_with_as_sync_str(&nested)?;
    nested.append_str("/file.txt");
    write_file(&nested, b"deep", true)?;

    // Removes the whole test tree, the files left by the other tests included
    Filesystem::remove_recursive(TEST_DIR)?;
    test_assert!(is_noent(Filesystem::stat(TEST_DIR)));

    // The siblings of the removed directory are untouched
    test_assert!(Filesystem::stat(FS_LOG_DIR).is_ok());
    Ok(())
}

pub(crate) fn run_all_tests(stats: &mut TestStats) {
    stats.record(TAG, "setup", setup());

    run_tests!(TAG, stats;
        test_stat_fs,
        test_system_dirs,
        test_open_missing,
        test_write_read_roundtrip,
        test_data_is_encrypted,
        test_truncate_on_rewrite,
        test_empty_file,
        test_sha256_integrity,
        test_seek_tell_size,
        test_read_error,
        test_write_error,
        test_closed_file,
        test_rename_remove,
        test_ls,
        // Last: it also cleans up TEST_DIR
        test_remove_recursive,
    );
}
