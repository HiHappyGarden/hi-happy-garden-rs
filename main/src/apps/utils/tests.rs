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


//! Utils tests: serial number generation and the encrypted JSON
//! persistence every config file goes through.
//!
//! Persistence tests write only `TEST_FILE` in `/var` and remove it (and
//! its `.sha256`) at the end.

use osal_rs::log_debug;
use osal_rs::os::RawMutex;
use osal_rs::utils::{Bytes, Result};
use osal_rs_serde::{Deserialize, Serialize};

use super::{CROCKFORD, deserialize_file, fnv1a64, luhn32_check, serial_from_uid, serialize_file};
use crate::drivers::filesystem::flags::{CREAT, TRUNC, WRONLY};
use crate::drivers::filesystem::{FileBytes, Filesystem};
use crate::drivers::platform::{FS_DATA_DIR, FS_SEPARATOR_DIR, Hardware};
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::hardware::HardwareFn;

const TAG: &str = "UtilsTests";

const TEST_FILE: &str = "hhg_utils_test.json";

static mut MUTEX: Option<RawMutex> = None;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct Doc {
    count: u8,
    offset: i16,
    port: u16,
    enabled: bool,
    name: Bytes<16>,
}

impl Default for Doc {
    fn default() -> Self {
        Self { count: 7, offset: -60, port: 123, enabled: true, name: Bytes::from_str("default") }
    }
}

fn mutex() -> &'static Option<RawMutex> {
    unsafe { &*&raw const MUTEX }
}

fn file_path(suffix: &str) -> FileBytes {
    let mut path = FileBytes::from_str(FS_DATA_DIR);
    path.append_str(FS_SEPARATOR_DIR);
    path.append_str(TEST_FILE);
    path.append_str(suffix);
    path
}

fn remove_test_files() {
    let _ = Filesystem::remove_with_as_sync_str(&file_path(""));
    let _ = Filesystem::remove_with_as_sync_str(&file_path(".sha256"));
}

fn setup() -> Result<()> {
    unsafe {
        MUTEX = Some(RawMutex::new()?);
    }
    remove_test_files();
    Ok(())
}

fn test_fnv1a64_vectors() -> Result<()> {
    // Reference vectors of the FNV-1a 64 bit specification
    test_assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    test_assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
    test_assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    Ok(())
}

fn test_serial_format() -> Result<()> {
    for uid in [[1u8, 2, 3, 4, 5, 6, 7, 8], [0u8; 8], [0xffu8; 8], Hardware::get_unique_id()] {
        let serial = serial_from_uid(&uid);
        log_debug!(TAG, "{:02x?} -> {}", uid, core::str::from_utf8(&serial).unwrap_or("?"));
        test_assert_eq!(serial[4], b'-');
        test_assert_eq!(serial[9], b'-');
        for (i, c) in serial.iter().enumerate() {
            if i != 4 && i != 9 {
                test_assert!(CROCKFORD.contains(c), "char {c} not in Crockford base32");
            }
        }
        test_assert_eq!(serial_from_uid(&uid), serial);
    }
    Ok(())
}

fn test_serial_known_values() -> Result<()> {
    // Pinned: a change here changes the serial of every board in the field
    test_assert_eq!(&serial_from_uid(&[1, 2, 3, 4, 5, 6, 7, 8]), b"FTTH-12SP-M");
    test_assert_eq!(&serial_from_uid(&[0; 8]), b"N33Z-GCH8-T");
    test_assert!(serial_from_uid(&[1, 2, 3, 4, 5, 6, 7, 8]) != serial_from_uid(&[1, 2, 3, 4, 5, 6, 7, 9]));
    Ok(())
}

fn test_luhn32_detects_typos() -> Result<()> {
    // Luhn mod N catches every single-character substitution
    let codes = [3u8, 14, 0, 31, 7, 22, 9, 18];
    let check = luhn32_check(&codes);
    for i in 0..codes.len() {
        for value in 0..32u8 {
            if value == codes[i] {
                continue;
            }
            let mut typo = codes;
            typo[i] = value;
            test_assert!(luhn32_check(&typo) != check, "typo at {i} -> {value} not detected");
        }
    }
    Ok(())
}

fn test_roundtrip() -> Result<()> {
    let doc = Doc { count: 42, offset: -330, port: 8_883, enabled: false, name: Bytes::from_str("garden") };
    serialize_file(mutex(), TAG, FS_DATA_DIR, TEST_FILE, &doc)?;
    let read: Doc = deserialize_file(mutex(), TAG, FS_DATA_DIR, TEST_FILE)?;
    test_assert_eq!(read, doc);
    Ok(())
}

fn test_missing_file_gives_default() -> Result<()> {
    remove_test_files();
    let read: Doc = deserialize_file(mutex(), TAG, FS_DATA_DIR, TEST_FILE)?;
    test_assert_eq!(read, Doc::default());
    // ...and the default is persisted for the next boot
    test_assert!(Filesystem::stat_with_as_sync_str(&file_path("")).is_ok());
    let again: Doc = deserialize_file(mutex(), TAG, FS_DATA_DIR, TEST_FILE)?;
    test_assert_eq!(again, Doc::default());
    Ok(())
}

fn test_invalid_json_gives_default() -> Result<()> {
    let mut file = Filesystem::open_with_as_sync_str(&file_path(""), WRONLY | CREAT | TRUNC)?;
    file.write(b"{ not json", true)?;
    drop(file);

    let read: Doc = deserialize_file(mutex(), TAG, FS_DATA_DIR, TEST_FILE)?;
    test_assert_eq!(read, Doc::default());
    Ok(())
}

fn test_tampered_file_gives_default() -> Result<()> {
    let doc = Doc { count: 1, offset: 1, port: 1, enabled: true, name: Bytes::from_str("tampered") };
    serialize_file(mutex(), TAG, FS_DATA_DIR, TEST_FILE, &doc)?;

    let mut sha = Filesystem::open_with_as_sync_str(&file_path(".sha256"), WRONLY | TRUNC)?;
    sha.write(&[b'f'; 64], false)?;
    drop(sha);

    let read: Doc = deserialize_file(mutex(), TAG, FS_DATA_DIR, TEST_FILE)?;
    test_assert_eq!(read, Doc::default());
    Ok(())
}

fn test_partial_json_keeps_defaults() -> Result<()> {
    // A file written by an older firmware lacks the newer fields
    let mut file = Filesystem::open_with_as_sync_str(&file_path(""), WRONLY | CREAT | TRUNC)?;
    file.write(br#"{"count":9}"#, true)?;
    drop(file);

    let read: Doc = deserialize_file(mutex(), TAG, FS_DATA_DIR, TEST_FILE)?;
    log_debug!(TAG, "partial: {:?}", read);
    // Either the whole file is rejected, or the missing fields keep their
    // defaults: zeroed fields (port 0, empty name) would break the device
    let merged = Doc { count: 9, ..Doc::default() };
    test_assert!(read == Doc::default() || read == merged, "partial file gave {read:?}");
    Ok(())
}

fn cleanup() -> Result<()> {
    remove_test_files();
    test_assert!(Filesystem::stat_with_as_sync_str(&file_path("")).is_err());
    Ok(())
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    stats.record(TAG, "setup", setup());

    run_tests!(TAG, stats;
        test_fnv1a64_vectors,
        test_serial_format,
        test_serial_known_values,
        test_luhn32_detects_typos,
        test_roundtrip,
        test_missing_file_gives_default,
        test_invalid_json_gives_default,
        test_tampered_file_gives_default,
        test_partial_json_keeps_defaults,
        cleanup,
    );
}
