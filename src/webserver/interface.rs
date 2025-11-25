use crate::{
    client::TimingSettings,
    database::PermanentlyStoredDataset,
    server::camera_program_types::{HeatData, HeatMeta},
    times::DayTime,
};
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
    UpdateTimingSettings(TimingSettings),
    RequestTimingSettings,
    Clock(DayTime),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayClientState {
    pub alive: bool,
    pub external_passthrough_mode: bool,
    pub can_switch_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageToWebControl {
    DisplayClientState(DisplayClientState),
    HeatsMeta(Vec<HeatMeta>),
    Logs(Vec<PermanentlyStoredDataset>),
    HeatDataMessage(HeatData),
    TimingSettingsState(TimingSettings),
}
