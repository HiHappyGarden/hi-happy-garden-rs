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

mod config;
mod display;
mod main;
mod parser;
mod screen_route;
mod session;
mod signals;
mod sprinkler;
mod system_handler;
mod system_led;
mod utils;
mod wifi;

pub(crate) use display::DISPLAY_INPUT_MAX_SIZE;
#[cfg_attr(feature = "tests", allow(unused_imports))]
pub(crate) use main::AppMain;


#[cfg(feature = "tests")]
mod test_helpers;

/// Runs the on-target application tests, see `crate::tests`.
///
/// The application layer is brought up first (signals, config, sprinkler)
/// the way `AppMain::init` does; if that fails nothing else can run.
#[cfg(feature = "tests")]
pub(crate) fn run_all_tests(stats: &mut crate::tests::TestStats) {
    let setup = test_helpers::setup();
    let ready = setup.is_ok();
    stats.record("AppsTests", "setup", setup);
    if !ready {
        return;
    }

    signals::tests::run_all_tests(stats);
    utils::tests::run_all_tests(stats);
    config::tests::run_all_tests(stats);
    session::tests::run_all_tests(stats);
    system_handler::tests::run_all_tests(stats);
    sprinkler::tests::run_all_tests(stats);
    main::tests::run_all_tests(stats);
    // Last: it starts the parser task, which then keeps running
    parser::tests::run_all_tests(stats);
}
