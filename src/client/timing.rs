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

pub enum TimingState {
    Stopped,
    Running(RaceTime),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TimingUpdate {
    Reset,
    Running(RaceTime),
    Intermediate(RaceTime),
    End(RaceTime),
}

pub struct TimingStateMachine {
    pub over_top_animation: Option<AnimationPlayer>,
    pub timing_state: TimingState,
    pub title: Option<String>,
    fireworks_animation: Animation,
    pub settings: TimingSettings,
}
impl TimingStateMachine {
    pub fn new(images_storage: &ImagesStorage, settings: &TimingSettings) -> TimingStateMachine {
        TimingStateMachine {
            over_top_animation: None,
            timing_state: TimingState::Stopped,
            title: None,
            fireworks_animation: images_storage.fireworks_animation.clone(), // animations can be lightweightly cloned
            settings: settings.clone(),
        }
    }

    pub fn update_race_time(&mut self, rtu: TimingUpdate) {
        match rtu {
            TimingUpdate::Reset => {
                self.timing_state = TimingState::Stopped;
            }
            TimingUpdate::Running(rt) => {
                self.timing_state = TimingState::Running(rt);
            }
            TimingUpdate::Intermediate(rt) => {
                self.timing_state = TimingState::Running(rt);

                if self.settings.fireworks_on_intermediate {
                    self.play_animation_over_top(AnimationPlayer::new(
                        &self.fireworks_animation,
                        false,
                    ));
                }
            }
            TimingUpdate::End(rt) => {
                self.timing_state = TimingState::Running(rt);

                if self.settings.fireworks_on_finish {
                    self.play_animation_over_top(AnimationPlayer::new(
                        &self.fireworks_animation,
                        false,
                    ));
                }
            }
        }
    }

    pub fn play_animation_over_top(&mut self, anim: AnimationPlayer) {
        self.over_top_animation = Some(anim);
    }

    pub fn overwrite_settings(&mut self, settings: &TimingSettings) {
        self.settings = settings.clone()
    }
}
