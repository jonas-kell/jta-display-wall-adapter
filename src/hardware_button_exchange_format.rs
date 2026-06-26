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

// TODO more general. Support button mapping and more controllers

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageFromHardwareButton {
    pub buttons_pressed: Vec<bool>,
}
impl MessageFromHardwareButton {
    fn parse_as_wall_controller(data: Self) -> WallControllerButtonsState {
        WallControllerButtonsState {
            blue: *data.buttons_pressed.get(0).unwrap_or(&false),
            green: *data.buttons_pressed.get(1).unwrap_or(&false),
            yellow: *data.buttons_pressed.get(2).unwrap_or(&false),
            white: *data.buttons_pressed.get(3).unwrap_or(&false),
            red: *data.buttons_pressed.get(4).unwrap_or(&false),
        }
    }
}

pub struct WallControllerButtonStateParser {
    previous_state: WallControllerButtonsState,
}
impl WallControllerButtonStateParser {
    pub fn new() -> Self {
        Self {
            previous_state: WallControllerButtonsState {
                blue: false,
                green: false,
                yellow: false,
                white: false,
                red: false,
            },
        }
    }

    pub fn parse_data_as_wall_controller(
        &mut self,
        mes: MessageFromHardwareButton,
    ) -> Option<WallControllerButtonEvent> {
        let parsed_state = MessageFromHardwareButton::parse_as_wall_controller(mes);
        let prev_state = &self.previous_state;

        let res = if !prev_state.blue && parsed_state.blue {
            Some(WallControllerButtonEvent::BluePressed)
        } else if !prev_state.green && parsed_state.green {
            Some(WallControllerButtonEvent::GreenPressed)
        } else if !prev_state.red && parsed_state.red {
            Some(WallControllerButtonEvent::RedPressed)
        } else if !prev_state.white && parsed_state.white {
            Some(WallControllerButtonEvent::WhitePressed)
        } else if !prev_state.yellow && parsed_state.yellow {
            Some(WallControllerButtonEvent::YellowPressed)
        } else {
            None
        };

        self.previous_state = parsed_state;

        return res;
    }
}

pub enum WallControllerButtonEvent {
    BluePressed,
    GreenPressed,
    YellowPressed,
    WhitePressed,
    RedPressed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WallControllerButtonsState {
    pub blue: bool,
    pub green: bool,
    pub yellow: bool,
    pub white: bool,
    pub red: bool,
}
