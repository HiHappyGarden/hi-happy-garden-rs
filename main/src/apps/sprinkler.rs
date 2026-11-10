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
use osal_rs::os::types::EventBits;
use osal_rs::os::{Mutex, MutexFn};
use osal_rs::utils::{OnceBox, Result};

use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::sprinkler::commons::Status;
use crate::apps::sprinkler::schedule::{ScheduleController, ScheduleZone};
use crate::apps::sprinkler::zone::{ZoneController, ZoneRelay};
use crate::drivers::date_time::DateTime;
use crate::traits::scheduler::OnSchedulerTrig;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;


const APP_TAG: &str = "AppSprinkler";

static SELECTED_SCHEDULE: AtomicUsize = AtomicUsize::new(0);

static TRIGGER_HANDLER: SprinklerTriggerHandler = SprinklerTriggerHandler::new();

static QUEUE_ZONES: OnceBox<Mutex<[ScheduleZone; ZoneController::SIZE]>> = OnceBox::new();

const DISBURSEMENT_MASK: EventBits = 
                StatusFlag::DisbursementZoneRelay0 as EventBits
                | StatusFlag::DisbursementZoneRelay1 as EventBits
                | StatusFlag::DisbursementZoneRelay2 as EventBits
                | StatusFlag::DisbursementZoneRelay3 as EventBits;

#[derive(Debug)]
pub(in crate::apps) struct Sprinkler {
    schedule_controller: &'static mut ScheduleController,
    zone_controller: &'static mut ZoneController,
}

impl Initializable for Sprinkler {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app sprinkler");
        
        QUEUE_ZONES.get_or_try_init(|| Ok(Mutex::new([
            ScheduleZone::new(ZoneRelay::Relay0),
            ScheduleZone::new(ZoneRelay::Relay1),
            ScheduleZone::new(ZoneRelay::Relay2),
            ScheduleZone::new(ZoneRelay::Relay3)
        ])))?;

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

        if (status & DISBURSEMENT_MASK) != 0 {
            return;
        }

        'main_loop: while let Some(schedule) = self.schedule_controller.into_iter().next() {
            if schedule.executable(&now) {
                if schedule.status == Status::RUN {
                    continue;
                }

                for (i, ScheduleZone { relay: zone_relay, minutes }) in schedule.into_iter().enumerate() {

                    let zone = &mut ZoneController::shared()[i];

                    if zone.zone_relay != zone_relay {
                        continue 'main_loop;
                    }

                    if minutes > 0 && zone.status == Status::ACTIVE {
                        let a = &mut QUEUE_ZONES.get().unwrap().lock().unwrap();
                        let zone_queue = &mut a[i];
                        zone_queue.minutes = minutes;
                    }
                }

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