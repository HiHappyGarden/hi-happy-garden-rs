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


//! Zone detail screen tests: construction and the transitions of its FSM.
//! Drawing needs an LCD and is not covered here.

use osal_rs::os::types::EventBits;
use osal_rs::utils::{Error, Result};

use super::{FSMState, ScreenZoneDetail};
use crate::apps::screen_route::step;
use crate::apps::signals::display::DisplaySignal;
use crate::apps::sprinkler::zone::ZoneController;
use crate::apps::test_helpers::{DRAW, with_draw_restored};
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::screen::{Answer, Nav, ScreenParam};
use crate::traits::signal::Signal;

const TAG: &str = "ZoneDetailTests";

fn test_new() -> Result<()> {
    for relay in 0..ZoneController::SIZE {
        let screen = ScreenZoneDetail::new(relay)?;
        test_assert_eq!(screen.zone_relay, relay);
        test_assert!(screen.fsm_state == FSMState::Description, "relay {relay} does not start on the description");
    }
    Ok(())
}

fn test_new_out_of_range() -> Result<()> {
    for relay in [ZoneController::SIZE, ZoneController::SIZE + 1, usize::MAX] {
        test_assert!(matches!(ScreenZoneDetail::new(relay), Err(Error::OutOfIndex)), "relay {relay} accepted");
    }
    Ok(())
}

fn test_step_pending() -> Result<()> {
    with_draw_restored(|| {
        let mut screen = ScreenZoneDetail::new(0)?;
        let mut display_signal: EventBits = 0;
        let nav = step(&mut screen, &mut display_signal, Answer::<u8, 2>::Pending, FSMState::Weight, FSMState::Status);
        test_assert!(matches!(nav, Nav::Stay));
        test_assert!(screen.fsm_state == FSMState::Description, "pending answer changed state");
        test_assert_eq!(display_signal, 0, "pending answer requested a redraw");
        Ok(())
    })
}

fn test_step_confirmed() -> Result<()> {
    with_draw_restored(|| {
        let mut screen = ScreenZoneDetail::new(0)?;
        let mut display_signal: EventBits = 0;
        let nav = step(&mut screen, &mut display_signal, Answer::<u8, 2>::Confirmed(ScreenParam::Number(3)), FSMState::Weight, FSMState::Status);
        test_assert!(matches!(nav, Nav::Stay));
        test_assert!(screen.fsm_state == FSMState::Weight, "confirm did not move to the next state");
        test_assert!(display_signal & DRAW != 0, "no redraw after confirm");
        test_assert!(DisplaySignal::get() & DRAW != 0, "display task not woken after confirm");
        Ok(())
    })
}

fn test_step_cancelled() -> Result<()> {
    with_draw_restored(|| {
        let mut screen = ScreenZoneDetail::new(0)?;
        screen.fsm_state = FSMState::Weight;
        let mut display_signal: EventBits = 0;
        let nav = step(&mut screen, &mut display_signal, Answer::<u8, 2>::Cancelled, FSMState::Status, FSMState::Description);
        test_assert!(matches!(nav, Nav::Stay));
        test_assert!(screen.fsm_state == FSMState::Description, "cancel did not move to the previous state");
        test_assert!(display_signal & DRAW != 0, "no redraw after cancel");
        Ok(())
    })
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_new,
        test_new_out_of_range,
        test_step_pending,
        test_step_confirmed,
        test_step_cancelled,
    );
}
