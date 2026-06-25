use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HardwareButtonStateBroadcast {
    pub buttons_active: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageFromHardwareButton {
    button_pressed: u8,
}
