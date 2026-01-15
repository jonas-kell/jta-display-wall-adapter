use rust_to_ts_types::TypescriptSerializable;
use serde::{Deserialize, Serialize};

use crate::client::{PUBLISH_FRAME_TIME_MESSAGE_EVERY_SECONDS, TARGET_FPS};

pub struct FrametimeTracker {
    counter: usize,
    reports_every_counter_at: usize,
    rolling_average_sum: u64,
    worst_1: u64,
    worst_2: u64,
    worst_3: u64,
    worst_4: u64,
    worst_5: u64,
}
impl FrametimeTracker {
    pub fn new() -> Self {
        Self {
            counter: 0,
            reports_every_counter_at: TARGET_FPS as usize
                * PUBLISH_FRAME_TIME_MESSAGE_EVERY_SECONDS as usize,
            rolling_average_sum: 0,
            worst_1: 0,
            worst_2: 0,
            worst_3: 0,
            worst_4: 0,
            worst_5: 0,
        }
    }

    pub fn digest_new_frame_time_percentage(&mut self, frame_time_percentage: u64) {
        self.counter += 1;
        self.rolling_average_sum += frame_time_percentage;

        if frame_time_percentage >= self.worst_1 {
            self.worst_5 = self.worst_4;
            self.worst_4 = self.worst_3;
            self.worst_3 = self.worst_2;
            self.worst_2 = self.worst_1;
            self.worst_1 = frame_time_percentage;
        } else if frame_time_percentage >= self.worst_2 {
            self.worst_5 = self.worst_4;
            self.worst_4 = self.worst_3;
            self.worst_3 = self.worst_2;
            self.worst_2 = frame_time_percentage;
        } else if frame_time_percentage >= self.worst_3 {
            self.worst_5 = self.worst_4;
            self.worst_4 = self.worst_3;
            self.worst_3 = frame_time_percentage;
        } else if frame_time_percentage >= self.worst_4 {
            self.worst_5 = self.worst_4;
            self.worst_4 = frame_time_percentage;
        } else if frame_time_percentage >= self.worst_5 {
            self.worst_5 = frame_time_percentage;
        }
    }

    pub fn needs_to_send_out_report(&mut self) -> Option<FrametimeReport> {
        if self.counter >= self.reports_every_counter_at {
            let res = Some(FrametimeReport {
                target_fps: TARGET_FPS,
                time_percentage_taken_per_frame_since_last_report: self.rolling_average_sum
                    / self.counter as u64,
                worst_n: [
                    self.worst_1,
                    self.worst_2,
                    self.worst_3,
                    self.worst_4,
                    self.worst_5,
                ]
                .into(),
            });

            self.reset();

            return res;
        } else {
            return None;
        }
    }

    fn reset(&mut self) {
        self.counter = 0;
        self.rolling_average_sum = 0;

        self.worst_1 = 0;
        self.worst_2 = 0;
        self.worst_3 = 0;
        self.worst_4 = 0;
        self.worst_5 = 0;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TypescriptSerializable)]
pub struct FrametimeReport {
    target_fps: u64,
    time_percentage_taken_per_frame_since_last_report: u64,
    worst_n: Vec<u64>,
}
