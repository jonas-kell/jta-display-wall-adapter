use crate::times::{DayTime, RaceTime, RaceWind};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub time: DayTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatFinish {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub time: DayTime,
    pub race_time: RaceTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatIntermediate {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub time: DayTime,
    pub intermediate_time_at: RaceTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatFalseStart {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeatStartList {
    pub name: String,
    pub id: Uuid,
    pub nr: u32,
    pub session_nr: u32,
    pub distance_meters: u32,
    pub scheduled_start_time: DayTime,
    pub competitors: Vec<HeatCompetitor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub wind: RaceWind,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatWindMissing {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompetitorEvaluated {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
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
    // usefull props
    pub name: String,
    pub distance_meters: u32,
    pub start_time: DayTime,
    pub wind: Option<RaceWind>,
    // vec data
    pub competitors_evaluated: Vec<HeatCompetitorResult>,
    pub competitors_left_to_evaluate: Vec<HeatCompetitor>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatMeta {
    pub id: Uuid,
    pub name: String,
    pub number: u32,
    pub scheduled_start_time_string: String,
}
impl From<HeatStartList> for HeatMeta {
    fn from(value: HeatStartList) -> Self {
        HeatMeta {
            id: value.id,
            name: value.name,
            number: value.nr,
            scheduled_start_time_string: value.scheduled_start_time.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatData {
    pub meta: HeatMeta,
    pub start_list: HeatStartList,
    pub start: Option<HeatStart>,
    pub intermediates: Option<Vec<HeatIntermediate>>,
    pub wind: Option<HeatWind>,
    pub finish: Option<HeatFinish>,
    pub evaluations: Option<Vec<CompetitorEvaluated>>,
    pub result: Option<HeatResult>,
}
