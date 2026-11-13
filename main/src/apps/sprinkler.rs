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

use core::mem::self;
use core::sync::atomic::AtomicUsize;
use core::time::Duration;

use alloc::vec::Vec;
use osal_rs::log_info;
use osal_rs::os::types::{EventBits, StackType, UBaseType};
use osal_rs::os::{BytesHasLen, QueueStreamed, System, Thread, ThreadFn};
use osal_rs::utils::{OnceBox, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::sprinkler::commons::Status;
use crate::apps::sprinkler::schedule::{Schedule, ScheduleController, ScheduleZone};
use crate::apps::sprinkler::zone::{Zone, ZoneController};
use crate::drivers::date_time::DateTime;
use crate::drivers::platform::ThreadPriority;
use crate::traits::scheduler::OnSchedulerTrig;
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;

const APP_TAG: &str = "AppSprinkler";

static SELECTED_SCHEDULE: AtomicUsize = AtomicUsize::new(0);

static TRIGGER_HANDLER: SprinklerTriggerHandler = SprinklerTriggerHandler::new();

static QUEUE_ZONES: OnceBox<QueueStreamed<SprinklerTriggerData>> = OnceBox::new();

const THREAD_NAME: &str = "app_sprinkler_thr";
const STACK_SIZE: StackType = 256;
const SLEEP_PERIOD: Duration = Duration::from_millis(100);

const DISBURSEMENT_MASK: EventBits = 
                StatusFlag::DisbursementZoneRelay0 as EventBits
                | StatusFlag::DisbursementZoneRelay1 as EventBits
                | StatusFlag::DisbursementZoneRelay2 as EventBits
                | StatusFlag::DisbursementZoneRelay3 as EventBits;

#[derive(Debug)]
pub(in crate::apps) struct Sprinkler {
    schedule_controller: &'static mut ScheduleController,
    zone_controller: &'static mut ZoneController,
    thread: Thread,
}

impl Initializable for Sprinkler {
    fn init(&mut self) -> Result<()> {
        log_info!(APP_TAG, "Init app sprinkler");
        
        QUEUE_ZONES.get_or_try_init(|| {
            QueueStreamed::<SprinklerTriggerData>::new(
                (ScheduleController::SIZE * ZoneController::SIZE) as UBaseType,
                SprinklerTriggerData::SERIALIZED_SIZE as UBaseType)
        })?;

        self.zone_controller.init()?;
        self.schedule_controller.init()?;

           
        self.thread = self.thread.spawn_simple(move || {
            loop {
                
                System::delay_with_to_tick(SLEEP_PERIOD);
            }
        })?;


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
            zone_controller: ZoneController::shared(),
            thread: Thread::new_with_to_priority(THREAD_NAME, STACK_SIZE, ThreadPriority::High)
        }
    }

    pub(in crate::apps) fn check(&mut self, now: DateTime) {

        let status = StatusSignal::get();

        if (status & DISBURSEMENT_MASK) != 0 {
            return;
        }

        'main_loop: while let Some(mut schedule) = self.schedule_controller.into_iter().next() {
            if schedule.executable(&now) {
                if schedule.status == Status::UNACTIVE {

                    if !Self::handle_schedule_not_running(&mut schedule, now) {
                        continue 'main_loop;
                    }

                }
                
                break;
            }
        }
    }

    pub(in crate::apps) const fn trigger_handler() -> &'static dyn OnSchedulerTrig {
        &TRIGGER_HANDLER
    }

    fn handle_schedule_not_running(schedule: &mut Schedule, now: DateTime) -> bool{
        //let zones = &mut QUEUE_ZONES.get().unwrap().lock().unwrap();

        let mut zones = Vec::<SprinklerTriggerData>::new();

        for (i, ScheduleZone { relay: zone_relay, minutes }) in schedule.into_iter().enumerate() {

            let zone = &mut ZoneController::shared()[i];

            if zone.zone_relay != zone_relay {
                return false;
            }

            if minutes > 0 && zone.status == Status::ACTIVE {
                zones.push(SprinklerTriggerData {
                    zone: zone.clone(),
                    start: now.clone(),
                    duration: minutes,
                    status: Status::UNACTIVE,
                    weight: zone.weight
                });
            }
        }

        zones.sort_by(|a, b| a.weight.cmp(&b.weight));

        schedule.status = Status::RUN;
        true
    }

}


#[derive(Clone, Debug, Serialize, Deserialize)]
struct SprinklerTriggerData {
    zone: Zone,
    start: DateTime,
    duration: u8,
    weight: u8,
    status: Status
}

impl BytesHasLen for SprinklerTriggerData {
    #[inline]
    fn len(&self) -> usize {
        Self::SERIALIZED_SIZE
    }
}

impl SprinklerTriggerData {
    /// Worst-case size on the osal-rs-serde binary wire format, used as fixed
    /// queue message size (`Bytes<N>` is variable: u32 length + up to N bytes).
    ///
    /// * `zone`: u32 len + description + zone_relay + weight + status
    /// * `start`: year(i32) + month, mday, wday, hour, minute, second (u8)
    ///   + millis(u16) + 2 bool
    /// * `duration` + `weight` + `status`
    const SERIALIZED_SIZE: usize =
        (mem::size_of::<u32>() + DISPLAY_INPUT_MAX_SIZE + 3)
        + (mem::size_of::<i32>() + 6 + mem::size_of::<u16>() + 2)
        + 3;
}



pub(in crate::apps) struct SprinklerTriggerHandler;

impl OnSchedulerTrig for SprinklerTriggerHandler {

    fn on_triggered(&self) -> Result<()> {
        // Implement the logic to handle the scheduler trigger here
        
        
        Ok(())
    }
}

impl SprinklerTriggerHandler {
    pub(in crate::apps) const fn new() -> Self {
        Self
    }
}