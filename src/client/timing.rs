use crate::{
    args::Args,
    client::FRAME_TIME_NS,
    interface::{
        ClientInternalMessageFromServerToClient::EmitTimingSettingsUpdate,
        MessageFromServerToClient,
    },
    server::{
        bib_detection::DisplayEntry,
        camera_program_types::{HeatResult, HeatStartList},
    },
    times::{DayTime, RaceTime},
};
use async_channel::{Sender, TrySendError};
use images_core::images::{Animation, AnimationPlayer, ImagesStorage};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TimingTimeDisplayMode {
    TimeBigAndHold,
    TimeBigAndHoldTop,
    TimeBigAndHoldWithRunName,
    TimeBigAndHoldTopWithRunName,
    StreetRun,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimingSettings {
    pub fireworks_on_intermediate: bool,
    pub fireworks_on_finish: bool,
    pub max_decimal_places_after_comma: i8,
    pub hold_time_ms: u32,
    pub display_time_ms: u32,
    pub play_sound_on_start: bool,
    pub play_sound_on_intermediate: bool,
    pub play_sound_on_finish: bool,
    pub can_currently_update_meta: bool,
    pub time_continues_running: bool,
    pub switch_to_start_list_automatically: bool,
    pub switch_to_timing_automatically: bool,
    pub switch_to_results_automatically: bool,
    pub mode: TimingTimeDisplayMode,
}
impl TimingSettings {
    pub fn new(args: &Args) -> Self {
        Self {
            fireworks_on_intermediate: args.fireworks_on_intermediate,
            fireworks_on_finish: args.fireworks_on_finish,
            max_decimal_places_after_comma: args.max_decimal_place_after_comma,
            hold_time_ms: args.hold_time_ms,
            display_time_ms: args.display_time_ms,
            play_sound_on_start: args.play_sound_on_start,
            play_sound_on_intermediate: args.play_sound_on_intermediate,
            play_sound_on_finish: args.play_sound_on_finish,
            can_currently_update_meta: true,
            time_continues_running: false,
            switch_to_start_list_automatically: true,
            switch_to_timing_automatically: true,
            switch_to_results_automatically: false,
            mode: TimingTimeDisplayMode::TimeBigAndHoldTopWithRunName,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeldTimeState {
    settings: TimingSettings,
    holding_start_time: Instant,
    pub held_at_m: Option<u32>,
    pub held_at_time: RaceTime,
}
impl HeldTimeState {
    fn holding_has_elapsed(&self) -> bool {
        let now = Instant::now();
        let diff = now.saturating_duration_since(self.holding_start_time);

        if diff.as_millis() > self.settings.hold_time_ms as u128 {
            return true;
        }

        return false;
    }
}

pub enum TimingState {
    Stopped,
    Running,
    Held,
    Finished(RaceTime),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TimingUpdate {
    StartList,
    Timing,
    ResultList,
    Meta(HeatStartList),
    ResultMeta(HeatResult),
    Reset,
    Running(RaceTime),
    Intermediate(RaceTime), // only ever produced by the camera program when sending manual intermediate signal (could never get the light barrier to emit it)
    End(RaceTime),          // always emmitted by the light barrier if active
}

// TODO could implement some mechanism to differentiate between relays and normal races over 1000m. This is also kind of unnecessary and probably would only ever matter for 3000m
pub enum RaceDistance {
    Sprint15Meters,
    Sprint20Meters,
    Sprint25Meters,
    Sprint30Meters,
    Sprint50Meters,
    Sprint60Meters,
    Sprint75Meters,
    Sprint80Meters,
    Sprint100Meters,
    Sprint110Meters,
    Sprint120Meters,
    Sprint150Meters,
    Sprint200Meters, // this is not really differentiable from 4x50m as the timing program exports distance and distanceType, but the camera program only re-exports distance
    Sprint300Meters, // this is not really differentiable from 4x75m as the timing program exports distance and distanceType, but the camera program only re-exports distance
    Sprint400Meters, // this is not really differentiable from 4x100m as the timing program exports distance and distanceType, but the camera program only re-exports distance
    Distance800Meters,
    Distance1000Meters,
    Distance1500Meters,
    Relay4x400Meters,
    Distance2000Meters,
    Relay3x800Meters,
    Distance3000Meters, // this is not really differentiable from 3x1000m as the timing program exports distance and distanceType, but the camera program only re-exports distance
    Distance5000Meters,
    Distance10000Meters,
    Custom(u32),
}
impl RaceDistance {
    pub fn new(distance: u32) -> Self {
        match distance {
            15 => Self::Sprint15Meters,
            20 => Self::Sprint20Meters,
            25 => Self::Sprint25Meters,
            30 => Self::Sprint30Meters,
            50 => Self::Sprint50Meters,
            60 => Self::Sprint60Meters,
            75 => Self::Sprint75Meters,
            80 => Self::Sprint80Meters,
            100 => Self::Sprint100Meters,
            110 => Self::Sprint110Meters,
            120 => Self::Sprint120Meters,
            150 => Self::Sprint150Meters,
            200 => Self::Sprint200Meters,
            300 => Self::Sprint300Meters,
            400 => Self::Sprint400Meters,
            800 => Self::Distance800Meters,
            1000 => Self::Distance1000Meters,
            1500 => Self::Distance1500Meters,
            1600 => Self::Relay4x400Meters,
            2000 => Self::Distance2000Meters,
            2400 => Self::Relay3x800Meters,
            3000 => Self::Distance3000Meters,
            5000 => Self::Distance5000Meters,
            10000 => Self::Distance10000Meters,
            other => Self::Custom(other),
        }
    }

    pub fn get_distance_as_number(&self) -> u32 {
        match self {
            Self::Sprint15Meters => 15,
            Self::Sprint20Meters => 20,
            Self::Sprint25Meters => 25,
            Self::Sprint30Meters => 30,
            Self::Sprint50Meters => 50,
            Self::Sprint60Meters => 60,
            Self::Sprint75Meters => 75,
            Self::Sprint80Meters => 80,
            Self::Sprint100Meters => 100,
            Self::Sprint110Meters => 110,
            Self::Sprint120Meters => 120,
            Self::Sprint150Meters => 150,
            Self::Sprint200Meters => 200,
            Self::Sprint300Meters => 300,
            Self::Sprint400Meters => 400,
            Self::Distance800Meters => 800,
            Self::Distance1000Meters => 1000,
            Self::Distance1500Meters => 1500,
            Self::Relay4x400Meters => 1600,
            Self::Distance2000Meters => 2000,
            Self::Relay3x800Meters => 2400,
            Self::Distance3000Meters => 3000,
            Self::Distance5000Meters => 5000,
            Self::Distance10000Meters => 10000,
            Self::Custom(other) => *other,
        }
    }

    // IWR 19.3
    // Die Zeiten aller im Ziel ankommenden Läufer sind zu erfassen. Zusätzlich müssen nach Möglichkeit auch die Rundenzeiten (des jeweils Führenden) bei Läufen von 800m und länger und die 1000m-Zeiten bei Läufen von 3000m und länger protokolliert werden.
    // Nationale Bestimmung DLV: Die Runden- bzw. Zwischenzeiten sind nur für den jeweils Führenden festzustellen und durch Hinzufügen seiner Startnummer in das Wettkampfprotokoll einzutragen. Dies gilt auch bei vollautomatischer Zeitnahme.

    pub fn get_split_distances(&self) -> Option<Vec<u32>> {
        match self {
            Self::Sprint15Meters => None,
            Self::Sprint20Meters => None,
            Self::Sprint25Meters => None,
            Self::Sprint30Meters => None,
            Self::Sprint50Meters => None,
            Self::Sprint60Meters => None,
            Self::Sprint75Meters => None,
            Self::Sprint80Meters => None,
            Self::Sprint100Meters => None,
            Self::Sprint110Meters => None,
            Self::Sprint120Meters => None,
            Self::Sprint150Meters => None,
            Self::Sprint200Meters => None,
            Self::Sprint300Meters => None,
            Self::Sprint400Meters => None,
            Self::Distance800Meters => Some([400].into()),
            Self::Distance1000Meters => Some([400, 800].into()),
            Self::Distance1500Meters => Some([400, 800, 1200].into()),
            Self::Relay4x400Meters => Some([400, 800, 1200].into()), // no idea, why this relay HAS intermediate times now.
            Self::Distance2000Meters => Some([400, 800, 1200, 1600].into()),
            Self::Relay3x800Meters => None, // seems to be exception in timing program -> 3x1000 also has None
            Self::Distance3000Meters => Some([1000, 2000].into()),
            Self::Distance5000Meters => Some([1000, 2000, 3000, 4000].into()),
            Self::Distance10000Meters => {
                Some([1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000].into())
            }
            Self::Custom(other) => {
                let mut res = Vec::new();

                if *other < 800 {
                    return None;
                }

                let round = if *other < 3000 { 400u32 } else { 1000u32 };

                let mut running: u32 = round;
                while running < *other {
                    res.push(running);
                    running += round;
                }

                Some(res)
            }
        }
    }

    fn get_time_continues_running(&self) -> bool {
        match self {
            Self::Sprint15Meters
            | Self::Sprint20Meters
            | Self::Sprint25Meters
            | Self::Sprint30Meters
            | Self::Sprint50Meters
            | Self::Sprint60Meters
            | Self::Sprint75Meters
            | Self::Sprint80Meters
            | Self::Sprint100Meters
            | Self::Sprint110Meters
            | Self::Sprint120Meters
            | Self::Sprint150Meters
            | Self::Sprint200Meters
            | Self::Sprint300Meters
            | Self::Sprint400Meters => false,
            Self::Distance800Meters
            | Self::Distance1000Meters
            | Self::Distance1500Meters
            | Self::Relay4x400Meters
            | Self::Distance2000Meters
            | Self::Relay3x800Meters
            | Self::Distance3000Meters
            | Self::Distance5000Meters
            | Self::Distance10000Meters => true,
            Self::Custom(other) => {
                if *other < 800 {
                    return false;
                } else {
                    return true;
                }
            }
        }
    }
}

pub struct TimingStateMeta {
    pub title: String,
    pub distance: RaceDistance,
}

#[derive(PartialEq, Eq, Clone)]
pub enum TimingMode {
    StartList,
    Timing,
    ResultList,
}

pub struct TimingStateMachine {
    fireworks_animation: Animation,
    pub over_top_animation: Option<AnimationPlayer>,
    pub meta: Option<TimingStateMeta>,
    pub settings: TimingSettings,
    time_held_counter: u16,
    pub timing_mode: TimingMode,
    timing_state: TimingState,
    held_time_state: Option<HeldTimeState>,
    reference_computation_time: Instant,
    client_state_machine_sender: Sender<MessageFromServerToClient>,
    race_finished: bool,
    run_display_entries: Vec<(i64, DisplayEntry)>,
}
impl TimingStateMachine {
    pub fn new(
        images_storage: &ImagesStorage,
        settings: &TimingSettings,
        client_state_machine_sender: Sender<MessageFromServerToClient>,
    ) -> TimingStateMachine {
        TimingStateMachine {
            over_top_animation: None,
            meta: None,
            time_held_counter: 0,
            fireworks_animation: images_storage.fireworks_animation.clone(), // animations can be lightweightly cloned
            settings: settings.clone(),
            timing_state: TimingState::Stopped,
            held_time_state: None,
            reference_computation_time: Instant::now(),
            client_state_machine_sender,
            race_finished: false,
            timing_mode: TimingMode::StartList,
            run_display_entries: Vec::new(),
        }
    }

    pub fn process_update(&mut self, rtu: TimingUpdate) {
        match rtu {
            TimingUpdate::Timing => {
                self.timing_mode = TimingMode::Timing;
            }
            TimingUpdate::StartList => {
                self.timing_mode = TimingMode::StartList;
            }
            TimingUpdate::ResultList => {
                self.timing_mode = TimingMode::ResultList;
            }
            TimingUpdate::Meta(hsl) => {
                if self.settings.can_currently_update_meta {
                    let rd = RaceDistance::new(hsl.distance_meters);
                    let time_continues_running = rd.get_time_continues_running();
                    self.meta = Some(TimingStateMeta {
                        title: hsl.name,
                        distance: rd,
                    });

                    // update settings
                    self.settings.time_continues_running = time_continues_running;

                    // update time_continues_running
                    match self.client_state_machine_sender.try_send(
                        MessageFromServerToClient::ClientInternal(EmitTimingSettingsUpdate(
                            self.settings.clone(),
                        )),
                    ) {
                        Ok(()) => {}
                        Err(e) => match e {
                            TrySendError::Full(_) => {
                                error!("Receivers are there, but inbound internal channel is full. This should not happen!");
                            }
                            TrySendError::Closed(_) => {
                                error!("Could not post message, internal channel closed unexpectedly. This is fatal, but we can not return from here. Expect program to shut down now");
                            }
                        },
                    }
                } else {
                    warn!("Race Meta update was trashed, because update is blocked by settings");
                }
                if self.settings.switch_to_start_list_automatically
                    && self.timing_mode != TimingMode::Timing
                {
                    // we do not want to switch BACK to start list from timing (e.g. on reset, as it sends both)
                    // but if we are further along, we are probably already on the fresh start of a new run
                    self.timing_mode = TimingMode::StartList;
                }
            }
            TimingUpdate::ResultMeta(hr) => {
                if self.settings.can_currently_update_meta {
                    let rd = RaceDistance::new(hr.distance_meters);
                    self.meta = Some(TimingStateMeta {
                        title: hr.name,
                        distance: rd,
                    });
                } else {
                    warn!("Race Result Meta update was trashed, because update is blocked by settings");
                }
                if self.settings.switch_to_results_automatically {
                    self.timing_mode = TimingMode::ResultList;
                }
            }
            TimingUpdate::Reset => {
                self.timing_state = TimingState::Stopped;
                self.held_time_state = None; // make sure, to clear this
                self.race_finished = false;
                self.time_held_counter = 0;

                // a reset gets sent with all start list requests, so we do not automatically switch anywhere
            }
            TimingUpdate::Running(rt) => {
                self.update_reference_computation_time(&rt);
                if self.settings.switch_to_timing_automatically
                    && self.timing_mode == TimingMode::StartList
                {
                    // can only switch to timing automatically from start list, not from result list
                    self.timing_mode = TimingMode::Timing;
                }

                match &self.timing_state {
                    TimingState::Stopped | TimingState::Running => {
                        self.timing_state = TimingState::Running;
                    }
                    TimingState::Held => {
                        if let Some(hts) = &self.held_time_state {
                            if hts.holding_has_elapsed() {
                                // keep holding, if camera program tries to interrupt holding prematurely
                                self.timing_state = TimingState::Running;
                            }
                        }
                    }
                    TimingState::Finished(_) => {
                        // this should not happen naturally. This can only happen, if you have selected time_continues_running=false
                        // then the race ends
                        // then you decide that the time should continue and you select time_continues_running=true
                        // this than would trigger the time to continue in this case
                        if self.settings.time_continues_running {
                            self.timing_state = TimingState::Running;
                        }
                    }
                };
            }
            TimingUpdate::Intermediate(rt) => {
                self.update_reference_computation_time(&rt);
                if self.settings.switch_to_timing_automatically
                    && self.timing_mode == TimingMode::StartList
                {
                    self.timing_mode = TimingMode::Timing;
                }

                if matches!(self.timing_state, TimingState::Finished(_)) {
                    if !self.settings.time_continues_running {
                        // if finished and time must stop, do not trigger intermediate holding
                        return;
                    }
                }

                // if we reach this location, hold time
                self.hold_time(rt);

                if self.settings.fireworks_on_intermediate {
                    self.play_animation_over_top(AnimationPlayer::new(
                        &self.fireworks_animation,
                        false,
                    ));
                }
            }
            TimingUpdate::End(rt) => {
                self.update_reference_computation_time(&rt);
                self.race_finished = true;
                if self.settings.switch_to_timing_automatically
                    && self.timing_mode == TimingMode::StartList
                {
                    self.timing_mode = TimingMode::Timing;
                }

                if self.settings.time_continues_running {
                    self.hold_time(rt);
                } else {
                    self.timing_state = TimingState::Finished(rt);
                }

                if self.settings.fireworks_on_finish {
                    self.play_animation_over_top(AnimationPlayer::new(
                        &self.fireworks_animation,
                        false,
                    ));
                }
            }
        }
    }

    fn hold_time(&mut self, race_time: RaceTime) {
        debug!("Time holding triggered");

        self.timing_state = TimingState::Held;
        self.time_held_counter += 1;

        let mut held_at_m = None; // TODO make possibility to re-set this and overwrite the split times etc.
        if let Some(meta) = &self.meta {
            if let Some(split_distances) = meta.distance.get_split_distances() {
                if self.time_held_counter as usize >= split_distances.len() + 1 {
                    held_at_m = Some(meta.distance.get_distance_as_number())
                } else {
                    if let Some(split_distance) =
                        split_distances.get(self.time_held_counter as usize - 1)
                    {
                        held_at_m = Some(*split_distance);
                    }
                }
            }
        }

        self.held_time_state = Some(HeldTimeState {
            settings: self.settings.clone(),
            holding_start_time: Instant::now(),
            held_at_m,
            held_at_time: race_time,
        });
    }

    /// also pushes the hold time state out, if necessary
    pub fn get_main_display_race_time(&mut self) -> RaceTime {
        match &self.timing_state {
            TimingState::Held => {
                match &self.held_time_state {
                    Some(hts) => {
                        if hts.holding_has_elapsed() {
                            // kick out earlier from holding than what we get from camera program
                            self.timing_state = TimingState::Running;
                        }
                    }
                    None => {
                        // this should not happen. On hold transition this always gets set
                        error!("Invalid state encountered. Attempting fix.");
                        self.timing_state = TimingState::Running;
                    }
                }
            }
            _ => {}
        };

        match &self.timing_state {
            TimingState::Stopped => RaceTime::get_zero_time(),
            TimingState::Running => self.get_currently_computed_race_time(),
            TimingState::Held => match &self.held_time_state {
                Some(hts) => hts.held_at_time.clone(),
                None => self.get_currently_computed_race_time(), // this should not happen. On hold transition this always gets set
            },
            TimingState::Finished(finish_time) => finish_time.clone(),
        }
    }

    pub fn race_finished(&self) -> bool {
        return self.race_finished;
    }

    fn update_reference_computation_time(&mut self, race_time: &RaceTime) {
        let ten_thousands = race_time.into_ten_thousands();

        if let Some(time) = Instant::now().checked_sub(Duration::from_micros(ten_thousands * 100)) {
            self.reference_computation_time = time;
        } else {
            error!("Could not correct reference computation time");
        }
    }

    fn get_currently_computed_race_time(&self) -> RaceTime {
        let diff = Instant::now().saturating_duration_since(self.reference_computation_time);
        diff.into()
    }

    pub fn get_held_display_race_time(&self) -> Option<HeldTimeState> {
        match &self.timing_state {
            TimingState::Stopped | TimingState::Finished(_) => None, // per definition on a real finish, no held time exists (otherwise it wouldbee held with race_finished = true)
            TimingState::Held | TimingState::Running => self.held_time_state.clone(),
        }
    }

    pub fn play_animation_over_top(&mut self, anim: AnimationPlayer) {
        self.over_top_animation = Some(anim);
    }

    pub fn overwrite_settings(&mut self, settings: &TimingSettings) {
        self.settings = settings.clone()
    }

    pub fn insert_new_display_entry(&mut self, entry: &DisplayEntry) {
        self.run_display_entries.push((
            self.settings.display_time_ms as i64 * 1000000 / FRAME_TIME_NS as i64,
            entry.clone(),
        ));
    }

    pub fn get_display_entries_at_lines_and_advance_frame_countdown(
        &mut self,
    ) -> (
        Option<DisplayEntry>,
        Option<DisplayEntry>,
        Option<DisplayEntry>,
    ) {
        for i in [0usize, 1, 2] {
            if let Some(entry) = self.run_display_entries.get_mut(i) {
                entry.0 -= 1;
            }
        }

        // TODO prettier withnot shifting upwards?
        let res = (
            self.run_display_entries.get(0).map(|a| a.1.clone()),
            self.run_display_entries.get(1).map(|a| a.1.clone()),
            self.run_display_entries.get(2).map(|a| a.1.clone()),
        );

        self.run_display_entries = self
            .run_display_entries
            .clone() // TOOD I think that could be done cleaner
            .into_iter()
            .filter(|(frames, _)| *frames >= 0)
            .collect();

        res
    }
}

pub struct ClockState {
    reference_computation_time: Instant,
    corresponding_clock_time: DayTime,
}
impl ClockState {
    pub fn new(clock_time: &DayTime) -> Self {
        Self {
            reference_computation_time: Instant::now(),
            corresponding_clock_time: clock_time.clone(),
        }
    }

    pub fn get_currently_computed_day_time(&self) -> DayTime {
        let diff = Instant::now().saturating_duration_since(self.reference_computation_time);
        self.corresponding_clock_time.add_duration(diff)
    }
}
