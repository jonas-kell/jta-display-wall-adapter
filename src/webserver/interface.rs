use crate::{database::PermanentlyStoredDataset, server::xml_types::HeatStart};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageFromWebControl {
    Idle,
    Advertisements,
    FreeText(String),
    RequestDisplayClientState,
    SwitchMode,
    GetHeatStarts,
    GetLogs(u32),
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
    HeatStarts(Vec<HeatStart>),
    Logs(Vec<PermanentlyStoredDataset>),
}
