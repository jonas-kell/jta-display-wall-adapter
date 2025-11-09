use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromServerToClient {
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromClientToServer {
    Unknown,
}
