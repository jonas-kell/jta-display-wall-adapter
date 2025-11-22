use crate::times::{DayTime, RaceTime, RaceWind};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum DisqualificationReason {
    Disqualified,
    DidNotStart,
    DidNotFinish,
    Canceled,
    Other(String),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct HeatStart {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub useless_heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    pub time: DayTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatFinish {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub useless_heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    pub time: DayTime,
    pub race_time: RaceTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatIntermediate {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub useless_heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    pub time: DayTime,
    pub intermediate_time_at: RaceTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatFalseStart {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub useless_heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatStartList {
    pub name: String,
    pub id: Uuid,
    pub useless_heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    pub nr: u32,
    pub session_nr: u32,
    pub useless_session_id: String, // this is sometimes a date, sometimes numerical -> I think we do not use this
    pub useless_event_id: String, // this is sometimes a uuid, sometimes numerical -> I think we do not use this
    pub distance_meters: u32,
    pub scheduled_start_time: DayTime,
    pub competitors: Vec<HeatCompetitor>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatCompetitor {
    pub id: String,
    pub lane: u32,
    pub bib: u32,
    pub class: String,
    pub last_name: String,
    pub first_name: String,
    pub nation: String,
    pub club: String,
    pub gender: String,
    pub disqualified: Option<DisqualificationReason>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatWind {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub useless_heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    pub useless_session_id: String, // this is sometimes a date, sometimes numerical -> I think we do not use this
    pub useless_event_id: String, // this is sometimes a uuid, sometimes numerical -> I think we do not use this
    pub wind: RaceWind,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompetitorEvaluated {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub useless_heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    pub useless_session_id: String, // this is sometimes a date, sometimes numerical -> I think we do not use this
    pub useless_event_id: String, // this is sometimes a uuid, sometimes numerical -> I think we do not use this
    pub competitor_result: HeatCompetitorResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DifferenceToCandidate {
    Winner,
    Difference(RaceTime),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatCompetitorResult {
    pub competitor: HeatCompetitor,
    pub distance: u32,
    pub rank: u32,
    pub runtime: RaceTime,
    pub runtime_full_precision: RaceTime,
    pub difference_to_winner: DifferenceToCandidate,
    pub difference_to_previous: DifferenceToCandidate,
    pub finish_time: DayTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatResult {
    pub id: Uuid,
    pub useless_heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    pub useless_session_id: String, // this is sometimes a date, sometimes numerical -> I think we do not use this
    pub useless_event_id: String, // this is sometimes a uuid, sometimes numerical -> I think we do not use this
    // usefull props
    pub name: String,
    pub distance_meters: u32,
    pub start_time: DayTime,
    pub wind: Option<RaceWind>,
    // vec data
    pub competitors_evaluated: Vec<HeatCompetitorResult>,
    pub competitors_left_to_evaluate: Vec<HeatCompetitor>,
}
