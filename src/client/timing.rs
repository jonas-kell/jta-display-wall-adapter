use std::time::{Duration, Instant};

use crate::{args::Args, times::RaceTime};
use images_core::images::{Animation, AnimationPlayer, ImagesStorage};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimingSettings {
    pub fireworks_on_intermediate: bool,
    pub fireworks_on_finish: bool,
    pub max_decimal_places_after_comma: i8,
}
impl TimingSettings {
    pub fn new(args: &Args) -> Self {
        Self {
            fireworks_on_intermediate: args.fireworks_on_intermediate,
            fireworks_on_finish: args.fireworks_on_finish,
            max_decimal_places_after_comma: args.max_decimal_place_after_comma,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeldTimeState {
    holding_start_time: Instant,
    pub held_at_m: Option<u32>,
    pub held_at_time: RaceTime,
    pub race_was_finished: bool,
}

pub enum TimingState {
    Stopped,
    Running,
    Held,
    Finished(RaceTime),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TimingUpdate {
    Reset,
    Running(RaceTime),
    Intermediate(RaceTime),
    End(RaceTime),
}

pub struct TimingStateMachine {
    fireworks_animation: Animation,
    pub over_top_animation: Option<AnimationPlayer>,
    pub title: Option<String>,
    pub settings: TimingSettings,
    timing_state: TimingState,
    held_time_state: Option<HeldTimeState>,
    reference_computation_time: Instant,
}
impl TimingStateMachine {
    pub fn new(images_storage: &ImagesStorage, settings: &TimingSettings) -> TimingStateMachine {
        TimingStateMachine {
            over_top_animation: None,
            title: None,
            fireworks_animation: images_storage.fireworks_animation.clone(), // animations can be lightweightly cloned
            settings: settings.clone(),
            timing_state: TimingState::Stopped,
            held_time_state: None,
            reference_computation_time: Instant::now(),
        }
    }

    pub fn update_race_time(&mut self, rtu: TimingUpdate) {
        match rtu {
            TimingUpdate::Reset => {
                self.timing_state = TimingState::Stopped;
            }
            TimingUpdate::Running(rt) => {
                self.update_reference_computation_time(&rt);
                self.timing_state = TimingState::Running;
            }
            TimingUpdate::Intermediate(rt) => {
                self.update_reference_computation_time(&rt);
                self.timing_state = TimingState::Held;
                self.held_time_state = Some(HeldTimeState {
                    holding_start_time: Instant::now(),
                    held_at_m: None, // TODO add logic
                    held_at_time: rt,
                    race_was_finished: false, // TODO add logic
                });

                if self.settings.fireworks_on_intermediate {
                    self.play_animation_over_top(AnimationPlayer::new(
                        &self.fireworks_animation,
                        false,
                    ));
                }
            }
            TimingUpdate::End(rt) => {
                self.update_reference_computation_time(&rt);
                self.timing_state = TimingState::Finished(rt);

                if self.settings.fireworks_on_finish {
                    self.play_animation_over_top(AnimationPlayer::new(
                        &self.fireworks_animation,
                        false,
                    ));
                }
            }
        }
    }

    pub fn get_main_display_race_time(&self) -> RaceTime {
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
        match &self.timing_state {
            TimingState::Finished(_) => {
                return true;
            }
            _ => {}
        };

        // TODO add case, that finished but still intermediates there

        return false;
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
        let diff = Instant::now().duration_since(self.reference_computation_time);

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
}
