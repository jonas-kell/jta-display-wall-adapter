use crate::hex::{parse_race_time, parse_two_digits, take_until_and_consume};
use chrono::NaiveDateTime;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::{IResult, Parser};

use crate::args::Args;
use crate::hex::hex_log_bytes;
use crate::instructions::{DayTime, RaceWind};
use crate::instructions::{InstructionFromCameraProgram, RaceTime};

use quick_xml::de::from_str;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

pub struct BufferedParserXML {
    state: Vec<u8>,
}
impl BufferedParserXML {
    pub fn new() -> Self {
        Self { state: Vec::new() }
    }

    pub fn feed_bytes(
        &mut self,
        packet: &[u8],
    ) -> Option<Result<InstructionFromCameraProgram, String>> {
        let termination_res = check_xml_termination_bytes(packet);

        if self.state.is_empty() {
            if termination_res {
                // self contained packet (never copy)
                return Some(decode_single_xml(&packet[..packet.len() - 1]));
            } else {
                self.state.extend_from_slice(&packet);
            }
        } else {
            if termination_res {
                self.state.extend_from_slice(&packet[..packet.len() - 1]);

                // self contained packet (never copy)
                let res = decode_single_xml(&self.state);

                // clear internal buffer
                self.state.clear();

                return Some(res);
            } else {
                // append full packet
                self.state.extend_from_slice(packet);
            }
        }

        // Missing data
        None
    }
}

// 0D seems to be end of a message, always
fn check_xml_termination_bytes(possible_packet: &[u8]) -> bool {
    possible_packet.ends_with(b"\x0D")
}

const DATETIME_FORMAT: &str = "%Y-%m-%d%H:%M:%S%.f";
#[derive(Clone)]
struct HeatDateTime(NaiveDateTime);

impl<'de> Deserialize<'de> for HeatDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, DATETIME_FORMAT)
            .map(HeatDateTime)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone)]
struct HeatTime(DayTime);

impl<'de> Deserialize<'de> for HeatTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DayTime::parse_from_string(&s)
            .map(HeatTime)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone)]
struct HeatRaceTime(RaceTime);

impl<'de> Deserialize<'de> for HeatRaceTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(
            RaceTime::parse_from_string(&s).map_err(serde::de::Error::custom)?,
        ))
    }
}

#[derive(Deserialize, Clone)]
struct HeatEventXML {
    #[serde(rename = "@Application")]
    application: String,
    #[serde(rename = "@Version")]
    version: String,
    #[serde(rename = "@Generated")]
    generated: HeatDateTime,
    #[serde(rename = "@Id")]
    id: Uuid,
    #[serde(rename = "@HeatId")]
    heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    #[serde(rename = "@Time")]
    time: Option<HeatTime>,
    #[serde(rename = "@Runtime")]
    runtime: Option<HeatRaceTime>,
    #[serde(rename = "@IsFalseStart")]
    is_false_start: Option<bool>,
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
impl TryFrom<HeatEventXML> for HeatStart {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        if let Some(time) = value.time {
            Ok(Self {
                application: value.application,
                generated: value.generated.0,
                useless_heat_id: value.heat_id,
                id: value.id,
                time: time.0,
                version: value.version,
            })
        } else {
            Err("No time value present".into())
        }
    }

    type Error = String;
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
impl TryFrom<HeatEventXML> for HeatFinish {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        if let Some(time) = value.time {
            if let Some(race_time) = value.runtime {
                Ok(Self {
                    application: value.application,
                    generated: value.generated.0,
                    useless_heat_id: value.heat_id,
                    id: value.id,
                    time: time.0,
                    version: value.version,
                    race_time: race_time.0,
                })
            } else {
                Err("No race time value present".into())
            }
        } else {
            Err("No time value present".into())
        }
    }

    type Error = String;
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
impl TryFrom<HeatEventXML> for HeatIntermediate {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        if let Some(time) = value.time {
            if let Some(race_time) = value.runtime {
                Ok(Self {
                    application: value.application,
                    generated: value.generated.0,
                    useless_heat_id: value.heat_id,
                    id: value.id,
                    time: time.0,
                    version: value.version,
                    intermediate_time_at: race_time.0,
                })
            } else {
                Err("No race time value present".into())
            }
        } else {
            Err("No time value present".into())
        }
    }

    type Error = String;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatFalseStart {
    pub application: String,
    pub version: String,
    pub generated: NaiveDateTime,
    pub id: Uuid,
    pub useless_heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
}
impl TryFrom<HeatEventXML> for HeatFalseStart {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        if let Some(is_false_start) = value.is_false_start {
            if is_false_start {
                Ok(Self {
                    application: value.application,
                    generated: value.generated.0,
                    useless_heat_id: value.heat_id,
                    id: value.id,
                    version: value.version,
                })
            } else {
                Err("IsFalseStart marker unexpectedly false".into())
            }
        } else {
            Err("IsFalseStart marker no present".into())
        }
    }

    type Error = String;
}

#[derive(Deserialize)]
struct HeatStartListXML {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@Id")]
    id: Uuid,
    #[serde(rename = "@HeatId")]
    heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    #[serde(rename = "@Nr")]
    nr: u32,
    #[serde(rename = "@SessionNr")]
    session_nr: u32,
    #[serde(rename = "@SessionId")]
    session_id: String,
    #[serde(rename = "@EventId")]
    event_id: String,
    #[serde(rename = "@DistanceMeters")]
    distance_meters: u32,
    #[serde(rename = "@ScheduledStarttime")]
    scheduled_start_time: HeatTime,
    #[serde(rename = "Startlist")]
    start_list: StartlistXML,
}

#[derive(Deserialize)]
struct StartlistXML {
    #[serde(default)]
    #[serde(rename = "Competitor")]
    competitors: Vec<HeatCompetitorXML>,
}

#[derive(Deserialize, Clone)]
struct HeatCompetitorXML {
    #[serde(rename = "@Id")]
    id: String,
    #[serde(rename = "@Lane")]
    lane: u32,
    #[serde(rename = "@Bib")]
    bib: u32,
    #[serde(rename = "@Class")]
    class: String,
    #[serde(rename = "@Lastname")]
    last_name: String,
    #[serde(rename = "@Firstname")]
    first_name: String,
    #[serde(rename = "@Nation")]
    nation: String,
    #[serde(rename = "@Club")]
    club: String,
    #[serde(rename = "@Gender")]
    gender: String,
    // optional data: result data
    #[serde(rename = "@Distance")]
    distance: Option<u32>,
    #[serde(rename = "@Rank")]
    rank: Option<u32>,
    #[serde(rename = "@Runtime")]
    runtime: Option<HeatRaceTime>,
    #[serde(rename = "@RuntimeFullPrecision")]
    runtime_full_precision: Option<HeatRaceTime>,
    #[serde(rename = "@DifferenceToWinner")]
    difference_to_winner: Option<String>,
    #[serde(rename = "@DifferenceToPrevious")]
    difference_to_previous: Option<String>,
    #[serde(rename = "@Finishtime")] // this is called differently in Evaluated!!
    finish_time: Option<HeatTime>,
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
impl From<HeatStartListXML> for HeatStartList {
    fn from(value: HeatStartListXML) -> Self {
        Self {
            name: value.name,
            id: value.id,
            useless_heat_id: value.heat_id,
            nr: value.nr,
            session_nr: value.session_nr,
            useless_session_id: value.session_id,
            useless_event_id: value.event_id,
            distance_meters: value.distance_meters,
            scheduled_start_time: value.scheduled_start_time.0,
            competitors: value
                .start_list
                .competitors
                .into_iter()
                .map(|c| c.into())
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatCompetitor {
    id: String,
    lane: u32,
    bib: u32,
    class: String,
    last_name: String,
    first_name: String,
    nation: String,
    club: String,
    gender: String,
}
impl From<HeatCompetitorXML> for HeatCompetitor {
    fn from(value: HeatCompetitorXML) -> Self {
        Self {
            id: value.id,
            lane: value.lane,
            bib: value.bib,
            class: value.class,
            last_name: value.last_name,
            first_name: value.first_name,
            nation: value.nation,
            club: value.club,
            gender: value.gender,
        }
    }
}

#[derive(Deserialize)]
struct HeatWindXML {
    #[serde(rename = "@Application")]
    application: String,
    #[serde(rename = "@Version")]
    version: String,
    #[serde(rename = "@Generated")]
    generated: HeatDateTime,
    #[serde(rename = "@Id")]
    id: Uuid,
    #[serde(rename = "@HeatId")]
    heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    #[serde(rename = "@SessionId")]
    session_id: String,
    #[serde(rename = "@EventId")]
    event_id: String,
    #[serde(rename = "@Wind")]
    wind: f32,
    #[serde(rename = "@WindUnit")]
    wind_unit: String,
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
impl TryFrom<HeatWindXML> for HeatWind {
    fn try_from(value: HeatWindXML) -> Result<Self, Self::Error> {
        if value.wind_unit != "MetersPerSecond" {
            return Err("Can only parse wind that is in unit 'MetersPerSecond'".into());
        }

        Ok(Self {
            id: value.id,
            useless_heat_id: value.heat_id,
            useless_session_id: value.session_id,
            useless_event_id: value.event_id,
            application: value.application,
            generated: value.generated.0,
            version: value.version,
            wind: RaceWind::parse_from_f32(value.wind),
        })
    }
    type Error = String;
}

#[derive(Deserialize)]
struct CompetitorEvaluatedXML {
    #[serde(rename = "@Application")]
    application: String,
    #[serde(rename = "@Version")]
    version: String,
    #[serde(rename = "@Generated")]
    generated: HeatDateTime,
    #[serde(rename = "@Id")]
    id: Uuid,
    #[serde(rename = "@HeatId")]
    heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    #[serde(rename = "@SessionId")]
    session_id: String,
    #[serde(rename = "@EventId")]
    event_id: String,
    // competitor data
    #[serde(rename = "@Lane")]
    lane: u32,
    #[serde(rename = "@Bib")]
    bib: u32,
    #[serde(rename = "@Class")]
    class: String,
    #[serde(rename = "@Lastname")]
    last_name: String,
    #[serde(rename = "@Firstname")]
    first_name: String,
    #[serde(rename = "@Nation")]
    nation: String,
    #[serde(rename = "@Club")]
    club: String,
    #[serde(rename = "@Gender")]
    gender: String,
    // competitor result data (must be there, here)
    #[serde(rename = "@Distance")]
    distance: u32,
    #[serde(rename = "@Rank")]
    rank: u32,
    #[serde(rename = "@Runtime")]
    runtime: HeatRaceTime,
    #[serde(rename = "@RuntimeFullPrecision")]
    runtime_full_precision: HeatRaceTime,
    #[serde(rename = "@DifferenceToWinner")]
    difference_to_winner: String,
    #[serde(rename = "@DifferenceToPrevious")]
    difference_to_previous: String,
    #[serde(rename = "@Time")] // this is called differently in general Competitor parser
    finish_time: HeatTime,
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
impl TryFrom<CompetitorEvaluatedXML> for CompetitorEvaluated {
    fn try_from(value: CompetitorEvaluatedXML) -> Result<Self, Self::Error> {
        let competitor_result = HeatCompetitorResult {
            competitor: HeatCompetitor {
                id: String::from("0"), // this is in line with the obesrved values in Results list (here as events, we do not know the id)
                lane: value.lane,
                bib: value.bib,
                class: value.class,
                last_name: value.last_name,
                first_name: value.first_name,
                nation: value.nation,
                club: value.club,
                gender: value.gender,
            },
            difference_to_previous: DifferenceToCandidate::parse_from_string(
                value.difference_to_previous,
            )?,
            difference_to_winner: DifferenceToCandidate::parse_from_string(
                value.difference_to_winner,
            )?,
            distance: value.distance,
            finish_time: value.finish_time.0,
            rank: value.rank,
            runtime: value.runtime.0,
            runtime_full_precision: value.runtime_full_precision.0,
        };

        Ok(Self {
            id: value.id,
            useless_heat_id: value.heat_id,
            useless_session_id: value.session_id,
            useless_event_id: value.event_id,
            application: value.application,
            generated: value.generated.0,
            version: value.version,
            competitor_result: competitor_result,
        })
    }
    type Error = String;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DifferenceToCandidate {
    Winner,
    Difference(RaceTime),
}
impl DifferenceToCandidate {
    fn parse_from_string(input: String) -> Result<DifferenceToCandidate, String> {
        if input == "Sieger" {
            return Ok(DifferenceToCandidate::Winner);
        } else {
            return RaceTime::parse_from_string(&input)
                .map(|i| DifferenceToCandidate::Difference(i));
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatCompetitorResult {
    competitor: HeatCompetitor,
    distance: u32,
    rank: u32,
    runtime: RaceTime,
    runtime_full_precision: RaceTime,
    difference_to_winner: DifferenceToCandidate,
    difference_to_previous: DifferenceToCandidate,
    finish_time: DayTime,
}
impl TryFrom<HeatCompetitorXML> for HeatCompetitorResult {
    fn try_from(value: HeatCompetitorXML) -> Result<Self, Self::Error> {
        let heat_competitor: HeatCompetitor = value.clone().into();

        if value.runtime_full_precision.is_none() {
            // does not yet have results
            return Err(Ok(heat_competitor));
        }

        // from here, we try to parse as with result!

        Ok(Self {
            competitor: heat_competitor,
            difference_to_previous: DifferenceToCandidate::parse_from_string(
                value
                    .difference_to_previous
                    .ok_or(Err(String::from("DifferenceToPrevious is not set")))?,
            )
            .map_err(|e| Err(e))?,
            difference_to_winner: DifferenceToCandidate::parse_from_string(
                value
                    .difference_to_winner
                    .ok_or(Err(String::from("DifferenceToWinner is not set")))?,
            )
            .map_err(|e| Err(e))?,
            distance: value
                .distance
                .ok_or(String::from("Distance is not set"))
                .map_err(|e| Err(e))?,
            finish_time: value
                .finish_time
                .ok_or(String::from("Finishtime is not set"))
                .map_err(|e| Err(e))?
                .0,
            rank: value
                .rank
                .ok_or(String::from("Rank is not set"))
                .map_err(|e| Err(e))?,
            runtime: value
                .runtime
                .ok_or(String::from("Runtime is not set"))
                .map_err(|e| Err(e))?
                .0,
            runtime_full_precision: value
                .runtime_full_precision
                .ok_or(String::from("RuntimeFullPrecision is not set"))
                .map_err(|e| Err(e))?
                .0,
        })
    }

    type Error = Result<HeatCompetitor, String>;
}

#[derive(Deserialize)]
struct HeatResultXML {
    #[serde(rename = "@Id")]
    id: Uuid,
    #[serde(rename = "@HeatId")]
    heat_id: String, // this is sometimes numerical, sometimes a uuid -> I think we do not use this
    #[serde(rename = "@SessionId")]
    session_id: String,
    #[serde(rename = "@EventId")]
    event_id: String,
    // usefull props
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@Starttime")]
    start_time: HeatTime,
    #[serde(rename = "@DistanceMeters")]
    distance_meters: u32,
    #[serde(rename = "@Wind")]
    wind: f32,
    #[serde(rename = "@WindUnit")]
    wind_unit: String,
    // array
    #[serde(rename = "Results")]
    results: ResultsXML,
}

#[derive(Deserialize)]
struct ResultsXML {
    #[serde(default)]
    #[serde(rename = "Competitor")]
    competitors: Vec<HeatCompetitorXML>,
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
    pub wind: RaceWind,
    // vec data
    pub competitors_evaluated: Vec<HeatCompetitorResult>,
    pub competitors_left_to_evaluate: Vec<HeatCompetitor>,
}
impl TryFrom<HeatResultXML> for HeatResult {
    fn try_from(value: HeatResultXML) -> Result<Self, Self::Error> {
        if value.wind_unit != "MetersPerSecond" {
            return Err("Can only parse wind that is in unit 'MetersPerSecond'".into());
        }

        let mut already_evaluated: Vec<HeatCompetitorResult> = Vec::new();
        let mut must_be_evaluated: Vec<HeatCompetitor> = Vec::new();

        for parse_data in value.results.competitors {
            match HeatCompetitorResult::try_from(parse_data) {
                Ok(res) => already_evaluated.push(res),
                Err(Err(e)) => return Err(e),
                Err(Ok(not_evaluated)) => must_be_evaluated.push(not_evaluated),
            }
        }

        Ok(Self {
            id: value.id,
            useless_heat_id: value.heat_id,
            useless_session_id: value.session_id,
            useless_event_id: value.event_id,
            wind: RaceWind::parse_from_f32(value.wind),
            distance_meters: value.distance_meters,
            name: value.name,
            start_time: value.start_time.0,
            competitors_evaluated: already_evaluated,
            competitors_left_to_evaluate: must_be_evaluated,
        })
    }

    type Error = String;
}

// https://docs.rs/quick-xml/latest/quick_xml/de/index.html
fn decode_single_xml(packet: &[u8]) -> Result<InstructionFromCameraProgram, String> {
    let decoded_string: String = String::from_utf8_lossy(packet).to_string();

    // the XML parser does not care for the root element name... Needs to be checked manually
    if decoded_string.starts_with("<HeatStart ")
        || decoded_string.starts_with("<HeatFinish ")
        || decoded_string.starts_with("<HeatIntermediate ")
    {
        match from_str::<HeatEventXML>(&decoded_string) {
            Ok(event) => {
                if decoded_string.starts_with("<HeatStart ") {
                    if let Ok(converted) = HeatFalseStart::try_from(event.clone()) {
                        return Ok(InstructionFromCameraProgram::HeatFalseStart(converted));
                    } else {
                        match HeatStart::try_from(event) {
                            Ok(converted) => {
                                return Ok(InstructionFromCameraProgram::HeatStart(converted))
                            }
                            Err(e) => return Err(format!("Must be start, could not parse: {}", e)),
                        }
                    }
                } else if decoded_string.starts_with("<HeatFinish ") {
                    match HeatFinish::try_from(event) {
                        Ok(converted) => {
                            return Ok(InstructionFromCameraProgram::HeatFinish(converted))
                        }
                        Err(e) => return Err(format!("Must be finish, could not parse: {}", e)),
                    }
                } else {
                    match HeatIntermediate::try_from(event) {
                        Ok(converted) => {
                            return Ok(InstructionFromCameraProgram::HeatIntermediate(converted))
                        }
                        Err(e) => {
                            return Err(format!("Must be intermediate, could not parse: {}", e))
                        }
                    }
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }

    if decoded_string.starts_with("<HeatStartlist ") {
        match from_str::<HeatStartListXML>(&decoded_string) {
            Ok(dec) => return Ok(InstructionFromCameraProgram::HeatStartList(dec.into())),
            Err(e) => return Err(e.to_string()),
        }
    }

    if decoded_string.starts_with("<HeatWind ") {
        match from_str::<HeatWindXML>(&decoded_string) {
            Ok(dec) => match HeatWind::try_from(dec) {
                Ok(dec) => return Ok(InstructionFromCameraProgram::HeatWind(dec)),
                Err(err) => return Err(format!("Wind could not be decoded: {}", err.to_string())),
            },
            Err(e) => return Err(e.to_string()),
        };
    }

    if decoded_string.starts_with("<HeatWind ") {
        match from_str::<HeatWindXML>(&decoded_string) {
            Ok(dec) => match HeatWind::try_from(dec) {
                Ok(dec) => return Ok(InstructionFromCameraProgram::HeatWind(dec)),
                Err(err) => return Err(format!("Wind could not be decoded: {}", err.to_string())),
            },
            Err(e) => return Err(e.to_string()),
        };
    }

    if decoded_string.starts_with("<CompetitorEvaluated ") {
        match from_str::<CompetitorEvaluatedXML>(&decoded_string) {
            Ok(dec) => match CompetitorEvaluated::try_from(dec) {
                Ok(dec) => return Ok(InstructionFromCameraProgram::CompetitorEvaluated(dec)),
                Err(err) => {
                    return Err(format!(
                        "Competitor Evaluation could not be decoded: {}",
                        err.to_string()
                    ))
                }
            },
            Err(e) => return Err(e.to_string()),
        };
    }

    if decoded_string.starts_with("<HeatResult ") {
        match from_str::<HeatResultXML>(&decoded_string) {
            Ok(dec) => match HeatResult::try_from(dec) {
                Ok(dec) => return Ok(InstructionFromCameraProgram::HeatResult(dec)),
                Err(err) => {
                    return Err(format!(
                        "Heat Result cloud not be decoded: {}",
                        err.to_string()
                    ))
                }
            },
            Err(e) => return Err(e.to_string()),
        };
    }

    error!("XML message fell through:\n{}", decoded_string);

    Err("Could not decode".into())
}

pub struct BufferedParserSerial {
    args: Args,
    state: Vec<u8>,
}
impl BufferedParserSerial {
    pub fn new(args: &Args) -> Self {
        Self {
            state: Vec::new(),
            args: args.clone(),
        }
    }

    pub fn feed_bytes(
        &mut self,
        packet: &[u8],
    ) -> Option<Result<InstructionFromCameraProgram, String>> {
        let termination_res = check_serial_termination_bytes(packet);

        if self.state.is_empty() {
            if termination_res {
                // self contained packet (never copy)
                return Some(decode_single_serial(
                    &self.args,
                    &packet[..packet.len() - 1],
                ));
            } else {
                self.state.extend_from_slice(&packet);
            }
        } else {
            if termination_res {
                self.state.extend_from_slice(&packet[..packet.len() - 1]);

                // self contained packet (never copy)
                let res = decode_single_serial(&self.args, &self.state);

                // clear internal buffer
                self.state.clear();

                return Some(res);
            } else {
                // append full packet
                self.state.extend_from_slice(packet);
            }
        }

        // Missing data
        None
    }
}

// 0D seems to be end of a message, always
fn check_serial_termination_bytes(possible_packet: &[u8]) -> bool {
    possible_packet.ends_with(b"\x0D")
}

fn decode_single_serial(
    args: &Args,
    packet: &[u8],
) -> Result<InstructionFromCameraProgram, String> {
    match parse_any_known_serial_command(packet) {
        Err(e) => trace!("Nom parser Error: {}", e.to_string()),
        Ok((_, command)) => return Ok(command),
    }

    if args.hexdump_incoming_communication {
        debug!("No serial command could be parsed from the following:");
        hex_log_bytes(packet);
    }

    Err("No matching command could be parsed for serial".into())
}

fn parse_clock_command(input: &[u8]) -> IResult<&[u8], InstructionFromCameraProgram> {
    let (input, _) = tag(&b"\x20\x20\x20\x20"[..])(input)?;
    let (input, _) = multispace0(input)?;
    let (input, hours) = parse_two_digits(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, minutes) = parse_two_digits(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, seconds) = parse_two_digits(input)?;
    let (input, _) = take_until_and_consume(&b"\x0D"[..], input)?;

    // TODO what is the second line?
    // It is not wind, I have never gotten it to deviate from the base value....

    // 202020202020202031363A31323A32382020202020200D4220202020202020202020202020203220202033202020

    let dt = DayTime {
        hours,
        minutes,
        seconds,
        fractional_part_in_ten_thousands: None,
    };

    Ok((input, InstructionFromCameraProgram::DayTime(dt)))
}

fn parse_zero_time_command(input: &[u8]) -> IResult<&[u8], InstructionFromCameraProgram> {
    const ZERO_TIME_START: [u8; 23] = [
        0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
        0x30, 0x2E, 0x30, 0x30, 0x30, 0x20, 0x20, 0x0D,
    ];
    const ZERO_TIME_START_ALTERNATIVE: [u8; 23] = [
        0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
        0x30, 0x2E, 0x30, 0x30, 0x20, 0x20, 0x20, 0x0D,
    ];

    alt((
        tag(&ZERO_TIME_START[..]),
        tag(&ZERO_TIME_START_ALTERNATIVE[..]),
    ))
    .parse(input)?;

    // starts the same way, as clock... I hate it. What does the byte really mean? :shrug:
    // 202020202020202020202020202020302E30303020200D4220202020202020202020202020203220202033202020
    // 202020202020202020202020202020302E30302020200D4220202020202020202020202020203220202033202020 // AHHH multiple points of precision??

    Ok((input, InstructionFromCameraProgram::ZeroTime))
}

fn parse_empty_time_command(input: &[u8]) -> IResult<&[u8], InstructionFromCameraProgram> {
    const EMPTY_TIME_START: [u8; 23] = [
        0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
        0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x0D,
    ];
    let (input, _) = tag(&EMPTY_TIME_START[..])(input)?;

    // 202020202020202020202020202020202020202020200D4220202020202020202020202020202020202020202020

    Ok((input, InstructionFromCameraProgram::ZeroTime))
}

fn parse_intermediate_time_command(input: &[u8]) -> IResult<&[u8], InstructionFromCameraProgram> {
    let (input, _) = tag(&b"\x20\x20\x20\x42"[..])(input)?;
    let (input, _) = multispace0(input)?;
    let (input, rt) = parse_race_time(input)?;
    let (input, _) = take_until_and_consume(&b"\x0D"[..], input)?;

    // 202020422020202031313A31393A35362E36353020200D4220202020202020202020202020203220202033202020

    Ok((input, InstructionFromCameraProgram::IntermediateTime(rt)))
}

fn parse_end_time_command(input: &[u8]) -> IResult<&[u8], InstructionFromCameraProgram> {
    let (input, _) = tag(&b"\x20\x20\x20\x43"[..])(input)?;
    let (input, _) = multispace0(input)?;
    let (input, rt) = parse_race_time(input)?;
    let (input, _) = take_until_and_consume(&b"\x0D"[..], input)?;

    // 202020432020202020202020202020342E32363620200D4220202020202020202020202020203220202033202020

    Ok((input, InstructionFromCameraProgram::EndTime(rt)))
}

fn parse_time_command(input: &[u8]) -> IResult<&[u8], InstructionFromCameraProgram> {
    let (input, _) = tag(&b"\x20\x20\x20\x2E"[..])(input)?;
    let (input, _) = multispace0(input)?;
    let (input, rt) = parse_race_time(input)?;
    let (input, _) = take_until_and_consume(&b"\x0D"[..], input)?;

    // 2020202E2020202020202020202020372E34202020200D4220202020202020202020202020203220202033202020
    // 2020202E2020202020202020353A30332E38202020200D4220202020202020202020202020203220202033202020
    // 2020202E2020202031303A33363A30302E32202020200D4220202020202020202020202020203220202033202020

    Ok((input, InstructionFromCameraProgram::RaceTime(rt)))
}

fn parse_any_known_serial_command(input: &[u8]) -> IResult<&[u8], InstructionFromCameraProgram> {
    alt((
        |i| parse_empty_time_command(i),
        |i| parse_zero_time_command(i),
        |i| parse_clock_command(i),
        |i| parse_intermediate_time_command(i),
        |i| parse_end_time_command(i),
        |i| parse_time_command(i),
    ))
    .parse(input)
}
