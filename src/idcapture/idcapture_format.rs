use crate::times::DayTime;
use rust_to_ts_types::TypescriptSerializable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, TypescriptSerializable)]
pub enum IDCaptureMessage {
    JumpToTime(DayTime),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageToIdcaptureServer {}
