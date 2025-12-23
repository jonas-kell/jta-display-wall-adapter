use std::collections::HashMap;

use crate::times::{DayTime, RaceTime, RaceWind};
use chrono::{NaiveDate, NaiveDateTime};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeatStart {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub time: DayTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeatFinish {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub time: DayTime,
    pub race_time: RaceTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub id: String, // not used for output to meetxml
    pub lane: u32,
    pub bib: u32,
    pub class: String,
    pub last_name: String,
    pub first_name: String,
    pub nation: String,
    pub club: String,
    pub gender: String,
    pub disqualified: Option<DisqualificationReason>, // not used for output to meetxml
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompetitorEvaluated {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub competitor_result: HeatCompetitorResult,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DifferenceToCandidate {
    Winner,
    Difference(RaceTime),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Meet {
    pub name: String,
    pub id: Uuid,
    pub city: String,
    pub sessions: Vec<Session>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub location: String,
    pub date: NaiveDate,
    pub events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DistanceType {
    Relay,
    Normal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub name: String,
    pub id: Uuid,
    pub distance: u32,
    pub distance_type: DistanceType,
    pub scheduled_start_time: DayTime,
    pub heats: Vec<Heat>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Heat {
    pub name: String,
    pub id: Uuid,
    pub distance: u32,
    pub scheduled_start_time: DayTime,
    pub distance_type: DistanceType,
    pub competitors: Vec<HeatCompetitor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Gender {
    Male,
    Female,
    Mixed,
}
impl Gender {
    pub fn to_string(&self) -> String {
        match self {
            Gender::Male => "M".into(),
            Gender::Female => "F".into(),
            Gender::Mixed => "X".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Athlete {
    pub id: Uuid,
    pub gender: Gender,
    pub bib: u32,
    pub club: String,
    pub first_name: String,
    pub last_name: String,
    pub nation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AthleteWithMetadata {
    pub athlete: Athlete,
    pub heats: Vec<(HeatAssignment, HeatData)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeatAssignment {
    pub id: i32,
    pub heat_id: Uuid,
    pub distance: u32,
    pub heat_descriminator: u8,
    pub athlete_ids: HashMap<u32, Uuid>,
}
