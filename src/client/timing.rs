use crate::times::RaceTime;
use images_core::images::{Animation, AnimationPlayer, ImagesStorage};
use serde::{Deserialize, Serialize};

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
}
impl TimingStateMachine {
    pub fn new(images_storage: &ImagesStorage) -> TimingStateMachine {
        TimingStateMachine {
            over_top_animation: None,
            timing_state: TimingState::Stopped,
            title: None,
            fireworks_animation: images_storage.fireworks_animation.clone(), // animations can be lightweightly cloned
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
            }
            TimingUpdate::End(rt) => {
                self.timing_state = TimingState::Running(rt);

                self.play_animation_over_top(AnimationPlayer::new(
                    &self.fireworks_animation,
                    false,
                ));
            }
        }
    }

    pub fn play_animation_over_top(&mut self, anim: AnimationPlayer) {
        self.over_top_animation = Some(anim);
    }
}
