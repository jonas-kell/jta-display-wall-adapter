use crate::{
    client::TimingSettings,
    database::PermanentlyStoredDataset,
    server::camera_program_types::{HeatData, HeatMeta},
    times::DayTime,
    wind::format::WindMeasurement,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageFromWebControl {
    Idle,
    Advertisements,
    FreeText(String),
    RequestDisplayClientState,
    SwitchMode,
    GetHeats,
    GetLogs(u32),
    SelectHeat(String),
    Timing,
    StartList,
    ResultList,
    UpdateTimingSettings(TimingSettings),
    RequestTimingSettings,
    Clock(DayTime),
    RequestWindValues(WindValueRequestDateContainer),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindValueRequestDateContainer {
    pub from: NaiveDateTime,
    pub to: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayClientState {
    pub alive: bool,
    pub external_passthrough_mode: bool,
    pub can_switch_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum MessageToWebControl {
    DisplayClientState(DisplayClientState),
    HeatsMeta(Vec<HeatMeta>),
    Logs(Vec<PermanentlyStoredDataset>),
    HeatDataMessage(HeatData),
    TimingSettingsState(TimingSettings),
    WindMeasurements(Vec<WindMeasurement>),
}
