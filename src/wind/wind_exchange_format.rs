use crate::times::{DayTime, RaceWind};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WindMessageBroadcast {
    Started(StartedWindMeasurement),
    Measured(WindMeasurement),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum WindMeasurementType {
    Polling,
    UnidentifiedMeasurement,
    Race10s,
    Race13s,
    Jump5s,
    Other8s,
    Other12s,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StartedWindMeasurement {
    pub ms_type: WindMeasurementType,
    pub time: Option<DayTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindMeasurement {
    pub wind: RaceWind,
    pub probable_measurement_type: WindMeasurementType,
    pub time: Option<DayTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageToWindServer {
    SetTime(DayTime),
}
