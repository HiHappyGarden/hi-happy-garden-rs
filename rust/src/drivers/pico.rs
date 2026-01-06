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
 
 #![allow(unused_imports)]

pub(crate) mod hardware;
pub(crate) mod gpio;
pub(crate) mod uart;

use core::ffi::c_char;
use osal_rs::os::types::ThreadHandle;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationMallocFailedHook() -> ! {
    // Hook for malloc failures - hang here for debugging
    #[allow(clippy::empty_loop)]
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationIdleHook() {
    // Idle hook - can be used for low power modes
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vApplicationStackOverflowHook(_x_task: ThreadHandle, _pc_task_name: *mut c_char) -> ! {
    // Stack overflow detected - hang here for debugging
    #[allow(clippy::empty_loop)]
    loop {}
}



