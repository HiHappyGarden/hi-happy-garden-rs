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

//! Hi Happy Garden firmware on the emulated Pico 2 W board.
//!
//! The host counterpart of `src/main.c`: prints the banner and hands control
//! to the firmware `start()`. The terminal is UART0, so the log shows up
//! here and AT commands are typed in. Built with the `tests` feature it runs
//! the firmware test suite instead and exits with 0 only if every test
//! passed.
//!
//! ```text
//! cargo run --no-default-features --features emulator,encryption --bin hhg-emulator -- --flash hhg-flash.bin
//! cargo run --no-default-features --features emulator,encryption,tests --bin hhg-emulator
//! ```

use std::process::ExitCode;

const USAGE: &str = "\
Usage: hhg-emulator [--flash <image>]

Options:
  --flash <image>  keep the flash partition in <image>, created blank if missing,
                   so configuration and data survive a restart (default: RAM only)
  -h, --help       print this help";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut flash_image = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--flash" => {
                let Some(path) = args.next() else {
                    eprintln!("--flash needs a file path\n\n{USAGE}");
                    return ExitCode::FAILURE;
                };
                flash_image = Some(path);
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}\n\n{USAGE}");
                return ExitCode::FAILURE;
            }
        }
    }

    if let Some(path) = flash_image
        && let Err(e) = ::main::emulator::set_flash_image(&path)
    {
        eprintln!("cannot use {path} as flash image: {e:?}");
        return ExitCode::FAILURE;
    }

    println!("===================================\r");
    println!("=== Hi Happy Garden RS {} ======\r", env!("CARGO_PKG_VERSION"));
    println!("===     emulated Pico 2 W     =====\r");
    println!("===================================\r\n\r");

    // Returns when the scheduler stops, on SIGINT or SIGTERM
    unsafe { ::main::start() };

    ExitCode::SUCCESS
}
