use crate::times::RaceTime;
use images_core::images::AnimationPlayer;
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
}
impl TimingStateMachine {
    pub fn new() -> TimingStateMachine {
        TimingStateMachine {
            over_top_animation: None,
            timing_state: TimingState::Stopped,
            title: None,
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

                // let anim = AnimationPlayer::new(
                //     &self.permanent_images_storage.fireworks_animation,
                //     self.frame_counter,
                //     false,
                // );

                // self.play_animation_over_top(anim);
            }
        }
    }

    pub fn play_animation_over_top(&mut self, anim: Option<AnimationPlayer>) {
        self.over_top_animation = anim;
    }
}
