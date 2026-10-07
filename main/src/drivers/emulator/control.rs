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

//! Control channel: the hands and eyes of whoever sits in front of the
//! emulated board.
//!
//! A Unix socket taking one text command per line and answering each with
//! one or more lines: press the buttons, turn the encoder, look at the
//! display, the relays and the LEDs, take the access point away. The
//! terminal stays the board UART, so the firmware log and the AT commands
//! are not mixed with these.
//!
//! ```text
//! scripts/hhg-emulator-cli.sh enc cw 3
//! echo display | socat - UNIX-CONNECT:/tmp/hhg-emulator.sock
//! ```

use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::Write as _;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::Duration;

use super::board;
use super::cyw43;
use super::gpio::{self, drive_input};
use super::sh1106;

/// Held long enough to get past the 50 ms debounce of the button drivers
const PRESS_TIME: Duration = Duration::from_millis(100);

/// Past the 300 ms long press of the display input handling
const LONG_PRESS_TIME: Duration = Duration::from_millis(800);

/// Between two encoder edges: past the 6 ms debounce, and long enough for
/// the encoder task to read the pins of each step
const ENCODER_EDGE_TIME: Duration = Duration::from_millis(20);

/// Upper bound of the steps a single `enc` command takes
const MAX_ENCODER_STEPS: u32 = 100;

pub(super) const HELP: &str = "\
commands:
  help                         this text
  status                       relays, RGB LED, CYW43 LED, WiFi
  display                      the screen, drawn with half blocks
  display pbm <file>           the screen saved as a PBM image
  btn click|press|release|long front panel button
  enc cw|ccw [steps]           turn the encoder, 1 step by default
  enc click|press|release|long encoder push button
  wifi up|down                 access point in or out of range
  gpio <pin> high|low|float    drive an input pin by hand";

/// Starts serving the control channel on the Unix socket at `path`.
///
/// A stale socket file left by an emulator that did not exit cleanly is
/// replaced; one still answered by a running emulator is an error.
pub(super) fn start(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        if UnixStream::connect(path).is_ok() {
            return Err(std::io::Error::new(ErrorKind::AddrInUse, "another emulator is serving this control socket"));
        }
        std::fs::remove_file(path)?;
    }

    let listener = UnixListener::bind(path)?;
    std::thread::Builder::new()
        .name("emu_control".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let _ = std::thread::Builder::new()
                    .name("emu_control_client".into())
                    .spawn(move || serve(stream));
            }
        })?;
    Ok(())
}

fn serve(stream: UnixStream) {
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    for line in BufReader::new(stream).lines() {
        let Ok(line) = line else {
            return;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let reply = match execute(line) {
            Ok(reply) => reply,
            Err(e) => format!("ERR {e}"),
        };
        if writeln!(writer, "{reply}").is_err() {
            return;
        }
    }
}

/// Runs one control command, returning its output.
pub(super) fn execute(line: &str) -> Result<String, String> {
    let mut words = line.split_whitespace();
    let command = words.next().unwrap_or("");
    let args: alloc::vec::Vec<&str> = words.collect();

    match (command, args.as_slice()) {
        ("help", []) => Ok(HELP.to_string()),
        ("status", []) => Ok(status()),
        ("display", []) => Ok(sh1106::render_text()),
        ("display", ["pbm", file]) => {
            std::fs::write(file, sh1106::render_pbm()).map_err(|e| format!("{file}: {e}"))?;
            Ok(format!("OK {file}"))
        }
        ("btn", [action]) => button(board::BTN, action),
        ("enc", ["cw" | "ccw"]) => rotate(args[0] == "cw", 1),
        ("enc", [direction @ ("cw" | "ccw"), steps]) => {
            let steps = steps.parse::<u32>().map_err(|_| format!("bad step count: {steps}"))?;
            if steps == 0 || steps > MAX_ENCODER_STEPS {
                return Err(format!("steps must be 1..={MAX_ENCODER_STEPS}"));
            }
            rotate(*direction == "cw", steps)
        }
        ("enc", [action]) => button(board::ENCODER_BTN, action),
        ("wifi", ["up"]) => {
            cyw43::set_ap_in_range(true);
            Ok("OK".to_string())
        }
        ("wifi", ["down"]) => {
            cyw43::set_ap_in_range(false);
            Ok("OK".to_string())
        }
        ("gpio", [pin, level]) => {
            let pin = pin.parse::<u32>().map_err(|_| format!("bad pin: {pin}"))?;
            let level = match *level {
                "high" => Some(true),
                "low" => Some(false),
                "float" => None,
                other => return Err(format!("bad level: {other}")),
            };
            drive_input(pin, level);
            Ok("OK".to_string())
        }
        _ => Err(format!("unknown command: {line} (try help)")),
    }
}

/// The front panel and encoder buttons are pull-up inputs, active low.
fn button(pin: u32, action: &str) -> Result<String, String> {
    let press = || drive_input(pin, Some(false));
    // Released means nothing drives the pin: the pull-up takes it high
    let release = || drive_input(pin, None);

    match action {
        "press" => press(),
        "release" => release(),
        "click" => {
            press();
            std::thread::sleep(PRESS_TIME);
            release();
        }
        "long" => {
            press();
            std::thread::sleep(LONG_PRESS_TIME);
            release();
        }
        other => return Err(format!("bad button action: {other} (click|press|release|long)")),
    }
    // Leaves the release past the debounce window before the next command
    std::thread::sleep(PRESS_TIME);
    Ok("OK".to_string())
}

/// Turns the encoder one detent at a time: two quadrature edges per detent,
/// the sequence the encoder driver decodes.
fn rotate(clockwise: bool, steps: u32) -> Result<String, String> {
    for _ in 0..steps * 2 {
        let ccw = gpio::level(board::ENCODER_CCW);
        let cw = gpio::level(board::ENCODER_CW);

        // Clockwise:          00 -> 01 -> 11 -> 10 -> 00 (bit 1 CCW, bit 0 CW)
        // Counter-clockwise:  00 -> 10 -> 11 -> 01 -> 00
        // Exactly one phase changes per edge: which one depends on the state
        let toggle_cw = (ccw == cw) == clockwise;
        if toggle_cw {
            drive_input(board::ENCODER_CW, Some(!cw));
        } else {
            drive_input(board::ENCODER_CCW, Some(!ccw));
        }
        std::thread::sleep(ENCODER_EDGE_TIME);
    }
    Ok("OK".to_string())
}

fn status() -> String {
    let mut out = String::new();

    let relays = board::RELAYS
        .iter()
        .enumerate()
        .map(|(zone, &pin)| format!("{}:{}", zone + 1, if gpio::output_level(pin) == Some(true) { "ON" } else { "off" }))
        .collect::<alloc::vec::Vec<_>>()
        .join(" ");
    let _ = writeln!(out, "relays     {relays}");

    let _ = writeln!(
        out,
        "rgb led    r:{} g:{} b:{} (PWM level, 0-255)",
        gpio::pwm_level(board::LED_RED),
        gpio::pwm_level(board::LED_GREEN),
        gpio::pwm_level(board::LED_BLUE)
    );
    let _ = writeln!(out, "cyw43 led  {}", if gpio::wl_gpio(board::CYW43_LED) { "ON" } else { "off" });
    let _ = writeln!(out, "wifi       {}", cyw43::describe());
    let _ = write!(out, "display    {} lit pixels", sh1106::lit_pixels());
    out
}
