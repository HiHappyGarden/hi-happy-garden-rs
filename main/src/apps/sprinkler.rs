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

#![allow(dead_code)]

#[cfg(feature = "tests")]
pub(super) mod tests;

pub(in crate::apps) mod commons;
pub(in crate::apps) mod zone;
pub(in crate::apps) mod schedule;

use core::sync::atomic::AtomicUsize;

use osal_rs::log_info;
use osal_rs::utils::Result;

use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::sprinkler::commons::Status;
use crate::apps::sprinkler::schedule::ScheduleController;
use crate::apps::sprinkler::zone::ZoneController;
use crate::drivers::date_time::DateTime;
use crate::traits::scheduler::OnSchedulerTrig;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;


const APP_TAG: &str = "AppSprinkler";

/// Index of the schedule returned by `query`, set via `AT+SPK=select,<index>`
static SELECTED_SCHEDULE: AtomicUsize = AtomicUsize::new(0);

static TRIGGER_HANDLER: SprinklerTriggerHandler = SprinklerTriggerHandler::new();

#[derive(Debug)]
pub(in crate::apps) struct Sprinkler {
    schedule_controller: &'static mut ScheduleController,
    zone_controller: &'static mut ZoneController,
}

impl Initializable for Sprinkler {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app sprinkler");
        
        self.zone_controller.init()?;
        self.schedule_controller.init()?;

           

        Ok(())
    }
}

impl Default for Sprinkler {

    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Sprinkler {
    const FILE_NAME: &'static str = "sprinkler.json";
    pub(in crate::apps) const AT_CMD: &'static str = "AT+SPK";
    pub(in crate::apps) const AT_RESP: &'static str = "+SPK: ";

    pub(in crate::apps) fn new() -> Self {
        Self {
            schedule_controller: ScheduleController::shared(),
            zone_controller: ZoneController::shared()
        }
    }

    pub(in crate::apps) fn check(&mut self, now: DateTime) {

        let status = StatusSignal::get();

        if StatusFlag::DisbursementInProgress.check_signal(status) {
            return;
        }

        for schedule in self.schedule_controller.into_iter() {
            if schedule.executable(&now) {
                StatusSignal::set(StatusFlag::DisbursementInProgress.into());
                schedule.status = Status::RUN;
                break;
            }
        }
    }

    pub(in crate::apps) const fn trigger_handler() -> &'static dyn OnSchedulerTrig {
        &TRIGGER_HANDLER
    }

}


pub(in crate::apps) struct SprinklerTriggerHandler;

impl OnSchedulerTrig for SprinklerTriggerHandler {

    fn on_triggered(&self) -> Result<()> {
        // Implement the logic to handle the scheduler trigger here
        
        log_info!(APP_TAG, "Scheduler triggered");
        
        Ok(())
    }
}

impl SprinklerTriggerHandler {
    pub(in crate::apps) const fn new() -> Self {
        Self
    }
}