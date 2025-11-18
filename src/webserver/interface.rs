use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageFromWebControlToWebSocket {
    Idle,
    Advertisements,
    FreeText(String),
}
