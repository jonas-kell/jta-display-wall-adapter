use crate::{database::PermanentlyStoredDataset, server::xml_types::HeatMeta};
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
}
