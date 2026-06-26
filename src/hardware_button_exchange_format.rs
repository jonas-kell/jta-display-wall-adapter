use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HardwareButtonStateBroadcast {
    pub buttons_light_on: Vec<bool>,
}
impl HardwareButtonStateBroadcast {
    pub fn wall_control(yellow: bool, green: bool, blue: bool, white: bool, red: bool) -> Self {
        Self {
            buttons_light_on: [blue, green, yellow, white, red].into(),
        }
    }
}

// TODO more general. Support buttn mapping and more controllers

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageFromHardwareButton {
    pub buttons_pressed: Vec<bool>,
}
impl MessageFromHardwareButton {
    pub fn parse_as_wall_controller(data: Self) -> WallControllerButtonsState {
        WallControllerButtonsState {
            blue: *data.buttons_pressed.get(0).unwrap_or(&false),
            green: *data.buttons_pressed.get(1).unwrap_or(&false),
            yellow: *data.buttons_pressed.get(2).unwrap_or(&false),
            white: *data.buttons_pressed.get(3).unwrap_or(&false),
            red: *data.buttons_pressed.get(4).unwrap_or(&false),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WallControllerButtonsState {
    pub blue: bool,
    pub green: bool,
    pub yellow: bool,
    pub white: bool,
    pub red: bool,
}
