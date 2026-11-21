/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2027 Antonio Salsi <passy.linux@zresa.it>
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
use osal_rs::os::{BytesHasLen, QueueStreamed, QueueStreamedFn, Thread, ThreadFn, ToTick};
use osal_rs::utils::{OnceBox, Result};
use osal_rs_serde::{Deserialize, Serialize};

use crate::apps::DISPLAY_INPUT_MAX_SIZE;
use crate::apps::signals::status::{StatusFlag, StatusSignal};
use crate::apps::sprinkler::commons::Status;
use crate::apps::sprinkler::schedule::{Schedule, ScheduleController, ScheduleZone};
use crate::apps::sprinkler::zone::{Zone, ZoneController, ZoneRelay};
use crate::drivers::date_time::DateTime;
use crate::drivers::gpio::Gpio;
use crate::drivers::platform::{GpioPeripheral, ThreadPriority};
use crate::traits::signal::Signal;
use crate::traits::state::Initializable;

const APP_TAG: &str = "AppSprinkler";

static SELECTED_SCHEDULE: AtomicUsize = AtomicUsize::new(0);

static QUEUE_ZONES: OnceBox<QueueStreamed<SprinklerTriggerData>> = OnceBox::new();

const THREAD_NAME: &str = "app_sprinkler_thr";
const STACK_SIZE: StackType = 1_024;
const SLEEP_PERIOD: Duration = Duration::from_millis(1_000);

pub (super) const DISBURSEMENT_MASK: EventBits = 
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

           
        self.thread = self.thread.spawn_simple( || {

            let mut sprinkler_trigger_data_vector: Vec::<SprinklerTriggerData> = Vec::<SprinklerTriggerData>::new();

            loop {
                let mut data = SprinklerTriggerData::default();
                let fetched = QUEUE_ZONES.get().unwrap().fetch(&mut data, SLEEP_PERIOD.to_ticks()).is_ok();

                if StatusSignal::get() & StatusFlag::DisbursementForcedStop as EventBits != 0 {
                    // the item just fetched (if any) is dropped together with the others
                    Self::forced_stop(&mut sprinkler_trigger_data_vector);
                    continue;
                }

                if fetched && data.status == Status::ACTIVE && data.duration > 0 {
                    sprinkler_trigger_data_vector.push(data);
                }

                // a full SLEEP_PERIOD has elapsed only when fetch timed out
                Self::step(&mut sprinkler_trigger_data_vector, !fetched);
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

        // re-arm the schedules that ran (or were stopped) once their time
        // window is over: inside it they would start again
        for schedule in self.schedule_controller.into_iter() {
            if schedule.status == Status::RUN && !schedule.matches(&now) {
                schedule.status = Status::ACTIVE;
            }
        }

        'main_loop: for schedule in self.schedule_controller.into_iter() {
            if schedule.executable(&now) {
                if schedule.status == Status::ACTIVE {

                    if !Self::handle_schedule_not_running(schedule, now) {
                        continue 'main_loop;
                    }

                }
                
                break;
            }
        }
    }


    fn handle_schedule_not_running(schedule: &mut Schedule, now: DateTime) -> bool{
        //let zones = &mut QUEUE_ZONES.get().unwrap().lock().unwrap();

        let mut sprinkler_trigger_data_vector = Vec::<SprinklerTriggerData>::new();

        for (i, ScheduleZone { relay: zone_relay, minutes }) in schedule.into_iter().enumerate() {

            let zone = &mut ZoneController::shared()[i];

            if zone.zone_relay != zone_relay {
                return false;
            }

            if minutes > 0 && zone.status == Status::ACTIVE {
                sprinkler_trigger_data_vector.push(SprinklerTriggerData {
                    zone: zone.clone(),
                    start: now.clone(),
                    duration: minutes as u16 * DateTime::SECONDS_PER_MINUTE as u16,
                    status: Status::ACTIVE,
                    weight: zone.weight
                });
            }
        }

        sprinkler_trigger_data_vector.sort_by(|a, b| a.weight.cmp(&b.weight));

        for data in sprinkler_trigger_data_vector.iter() {
            let _ = QUEUE_ZONES.get().unwrap().post(&data.clone(), Duration::from_millis(100).to_ticks());
        }

        schedule.status = Status::RUN;
        true
    }

    /// Advances the disbursement FIFO by one loop: only its head is watered.
    /// A finished zone is removed and the next one is opened in the same
    /// step, so `DISBURSEMENT_MASK` never drops to zero in between (`check`
    /// relies on it) and two valves are never open together.
    fn step(fifo: &mut Vec<SprinklerTriggerData>, elapsed: bool) {
        let Some(current) = fifo.first_mut() else {
            return;
        };

        if current.status == Status::ACTIVE {
            current.status = Status::RUN;
            Self::set_relay(current.zone.zone_relay, true);
            return;
        }

        if elapsed {
            current.duration = current.duration.saturating_sub(SLEEP_PERIOD.as_secs() as u16);
        }

        if current.duration > 0 {
            return;
        }

        let finished = fifo.remove(0).zone.zone_relay;

        match fifo.first_mut() {
            Some(next) => {
                next.status = Status::RUN;
                if next.zone.zone_relay != finished {
                    // raise the next flag first: no gap in DISBURSEMENT_MASK
                    StatusSignal::set(Self::relay_flag(next.zone.zone_relay) as EventBits);
                    Self::set_relay(finished, false);
                    Self::set_relay(next.zone.zone_relay, true);
                }
            }
            None => Self::set_relay(finished, false),
        }
    }

    #[inline]
    fn relay_flag(relay: ZoneRelay) -> StatusFlag {
        match relay {
            ZoneRelay::Relay0 => StatusFlag::DisbursementZoneRelay0,
            ZoneRelay::Relay1 => StatusFlag::DisbursementZoneRelay1,
            ZoneRelay::Relay2 => StatusFlag::DisbursementZoneRelay2,
            ZoneRelay::Relay3 => StatusFlag::DisbursementZoneRelay3,
        }
    }

    /// Drives one zone relay and its `Disbursement*` status flag together,
    /// so the flags always mirror the GPIO outputs.
    fn set_relay(relay: ZoneRelay, on: bool) {
        let flag = Self::relay_flag(relay);
        let gpio = match relay {
            ZoneRelay::Relay0 => GpioPeripheral::Relay0,
            ZoneRelay::Relay1 => GpioPeripheral::Relay1,
            ZoneRelay::Relay2 => GpioPeripheral::Relay2,
            ZoneRelay::Relay3 => GpioPeripheral::Relay3,
        };

        if on {
            StatusSignal::set(flag as EventBits);
            Gpio::shared().write(&gpio, 1);
        } else {
            Gpio::shared().write(&gpio, 0);
            StatusSignal::clear(flag as EventBits);
        }
    }

    /// Handles [`StatusFlag::DisbursementForcedStop`]: closes every relay,
    /// drops the running and the still queued zones, then acknowledges the
    /// request by clearing the flag.
    fn forced_stop(running: &mut Vec<SprinklerTriggerData>) {
        log_info!(APP_TAG, "Disbursement forced stop");

        for relay in [ZoneRelay::Relay0, ZoneRelay::Relay1, ZoneRelay::Relay2, ZoneRelay::Relay3] {
            Self::set_relay(relay, false);
        }

        running.clear();

        let queue = QUEUE_ZONES.get().unwrap();
        let mut data = SprinklerTriggerData::default();
        while queue.fetch(&mut data, 0).is_ok() {}

        StatusSignal::clear(StatusFlag::DisbursementForcedStop as EventBits);
    }

}


#[derive(Clone, Debug, Serialize, Deserialize)]
struct SprinklerTriggerData {
    zone: Zone,
    start: DateTime,
    duration: u16,
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
    /// queue message size. `size_of::<Self>()` is not enough: `Bytes<N>` is
    /// sent as u32 length + up to N bytes.
    ///
    /// * `zone`: u32 len + description + zone_relay + weight + status (u8)
    /// * `start`: year (i32) + month, mday, wday, hour, minute, second (u8)
    ///   + millis (u16) + 2 bool
    /// * `duration` (u16) + `weight` + `status` (u8)
    const SERIALIZED_SIZE: usize =
        (mem::size_of::<u32>() + DISPLAY_INPUT_MAX_SIZE + 3)
        + (mem::size_of::<i32>() + 6 + mem::size_of::<u16>() + 2)
        + (mem::size_of::<u16>() + 2);
}

impl Default for SprinklerTriggerData {
    fn default() -> Self {
        Self {
            zone: Zone::new(ZoneRelay::Relay0),
            start: DateTime::default(),
            duration: 0,
            weight: 0,
            status: Status::UNACTIVE,
        }
    }
}
