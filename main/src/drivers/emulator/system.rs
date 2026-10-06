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

//! Chip level services: unique id, reset, POWMAN timer, SHA-256
//! accelerator, repeating timers and the SysTick handshake.
//!
//! Twin of `src/pico/hhg-system-wrapper.c`, `hhg-timers.c` and
//! `hhg-systick.c`.

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::ffi::{c_int, c_uchar, c_ulonglong, c_void};
use core::sync::atomic::{AtomicBool, Ordering};
use std::os::unix::process::CommandExt;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

use crate::drivers::encrypt::SHA256_RESULT_BYTES;

use super::lock;

/// Board unique id of the emulator: fixed, because the filesystem AES key is
/// derived from it and a flash image must stay readable on any host
const UNIQUE_ID: [u8; 8] = *b"HHG-EMU1";

/// POWMAN timer: the value last set and when it was set; stopped (0) at power on
static POWMAN: Mutex<Option<(u64, Instant)>> = Mutex::new(None);

/// Always 1: there is no SysTick to set up, osal-rs `posix` ticks on the
/// host monotonic clock.
pub(crate) unsafe fn get_g_setup_called() -> u32 {
    1
}

pub(crate) unsafe fn print_systick_status() {
    osal_rs::println!("=== SysTick: emulated, osal-rs posix ticks on the host monotonic clock ===");
}

pub(in crate::drivers) unsafe fn hhg_get_unique_id(id_buffer: *mut u8) {
    if !id_buffer.is_null() {
        unsafe { core::ptr::copy_nonoverlapping(UNIQUE_ID.as_ptr(), id_buffer, UNIQUE_ID.len()) };
    }
}

/// Restarts the firmware: the emulator process replaces itself with a fresh
/// copy, as the watchdog reboot does on the board.
///
/// The flash image file, when given, keeps its content across the restart
/// just like the real flash.
pub(in crate::drivers) unsafe fn hhg_system_reset() -> ! {
    use std::io::Write;
    let _ = std::io::stdout().flush();

    let mut args = std::env::args_os();
    if let (Ok(exe), Some(_)) = (std::env::current_exe(), args.next()) {
        // exec only returns on failure
        let err = std::process::Command::new(exe).args(args).exec();
        std::eprintln!("emulator: reset failed: {err}");
    }
    std::process::exit(1)
}

pub(in crate::drivers) unsafe fn hhg_pico_sha256_start_blocking(state: *mut *mut c_void, _use_dma: bool) -> c_int {
    if state.is_null() {
        return -1;
    }
    unsafe { *state = Box::into_raw(Box::new(Sha256::new())) as *mut c_void };
    0
}

pub(in crate::drivers) unsafe fn hhg_pico_sha256_update_blocking(state: *mut c_void, data: *const c_uchar, data_size_bytes: usize) {
    let Some(hasher) = (unsafe { (state as *mut Sha256).as_mut() }) else {
        return;
    };
    if data.is_null() || data_size_bytes == 0 {
        return;
    }
    hasher.update(unsafe { core::slice::from_raw_parts(data, data_size_bytes) });
}

pub(in crate::drivers) unsafe fn hhg_pico_sha256_finish(state: *mut c_void, out: *mut c_uchar) {
    if state.is_null() {
        return;
    }
    // Takes the state back: it is freed here, as the C wrapper does
    let hasher = unsafe { Box::from_raw(state as *mut Sha256) };
    let digest = hasher.finalize();
    if !out.is_null() {
        unsafe { core::ptr::copy_nonoverlapping(digest.as_ptr(), out, SHA256_RESULT_BYTES) };
    }
}

pub(in crate::drivers) unsafe fn hhg_powman_timer_set_ms(time_ms: c_ulonglong) {
    *lock(&POWMAN) = Some((time_ms, Instant::now()));
}

pub(in crate::drivers) unsafe fn hhg_powman_timer_get_ms() -> c_ulonglong {
    match *lock(&POWMAN) {
        Some((set_ms, set_at)) => set_ms + set_at.elapsed().as_millis() as u64,
        None => 0,
    }
}

/// Handle of a repeating timer, what `repeating_timer_t` is on the board
struct RepeatingTimer {
    cancelled: Arc<AtomicBool>,
}

/// The callback argument crosses into the timer thread as an address: the
/// firmware owns what it points to, exactly as with the pico-sdk alarm pool
struct UserData(usize);

pub(in crate::drivers) unsafe fn hhg_add_repeating_timer_ms(delay_ms: c_int, callback: extern "C" fn(*mut c_void), user_data: *mut c_void, out: *mut *mut c_void) -> bool {
    if out.is_null() {
        return false;
    }

    // Same contract as the C wrapper: a handle already in `out` is released
    let previous = unsafe { *out };
    if !previous.is_null() {
        drop(unsafe { Box::from_raw(previous as *mut RepeatingTimer) });
    }

    let cancelled = Arc::new(AtomicBool::new(false));
    let period = Duration::from_millis(delay_ms.unsigned_abs() as u64);
    let user_data = UserData(user_data as usize);

    let thread_cancelled = cancelled.clone();
    let spawned = std::thread::Builder::new()
        .name("emu_hw_timer".into())
        .spawn(move || {
            // pico-sdk: a negative delay counts from the start of the
            // previous callback, a positive one from its end
            let mut next = Instant::now() + period;
            loop {
                let now = Instant::now();
                if next > now {
                    std::thread::sleep(next - now);
                }
                if thread_cancelled.load(Ordering::Acquire) {
                    return;
                }
                let started = Instant::now();
                callback(user_data.0 as *mut c_void);
                next = if delay_ms < 0 { started + period } else { Instant::now() + period };
            }
        });

    if spawned.is_err() {
        unsafe { *out = core::ptr::null_mut() };
        return false;
    }

    unsafe { *out = Box::into_raw(Box::new(RepeatingTimer { cancelled })) as *mut c_void };
    true
}

pub(in crate::drivers) unsafe fn hhg_cancel_repeating_timer(timer: *mut c_void) -> bool {
    if timer.is_null() {
        return false;
    }
    let timer = unsafe { Box::from_raw(timer as *mut RepeatingTimer) };
    !timer.cancelled.swap(true, Ordering::AcqRel)
}
