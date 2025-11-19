use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageFromWebControl {
    Idle,
    Advertisements,
    FreeText(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageToWebControl {
    DisplayClientAlive,
}
