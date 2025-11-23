use crate::args::Args;
use crate::hex::hex_log_bytes;
use crate::hex::{parse_race_time, parse_two_digits, take_until_and_consume};
use crate::instructions::InstructionFromCameraProgram;
use crate::server::camera_program_datatypes::{
    CompetitorEvaluated, DifferenceToCandidate, DisqualificationReason, HeatCompetitor,
    HeatCompetitorResult, HeatFalseStart, HeatFinish, HeatIntermediate, HeatResult, HeatStart,
    HeatStartList, HeatWind,
};
use crate::server::xml_types::HeatWindMissing;
use crate::times::{DayTime, RaceTime, RaceWind};
use chrono::NaiveDateTime;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::{IResult, Parser};
use quick_xml::de::from_str;
use serde::{Deserialize, Deserializer};
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

impl DisqualificationReason {
    fn parse_from_string(input: &str) -> Self {
        match input {
            "DISQ." | "DISQ" | "disqualifiziert" | "DSQ" | "DQ" => Self::Disqualified,
            "Nicht im Ziel" | "aufgegeben" | "gestürzt" | "DNF" | "surrender" | "fall" => {
                Self::DidNotFinish
            }
            "DNS" | "Nicht am Start" => Self::DidNotStart,
            "abgemeldet" | "checkout" | "CAN" => Self::Canceled,
            other => Self::Other(String::from(other)),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct HeatEventXML {
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
impl TryFrom<HeatEventXML> for HeatStart {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        let _ = value.heat_id; // drop because we get inconsistent type from source

        if let Some(time) = value.time {
            Ok(Self {
                application: value.application,
                generated: value.generated.0,
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
impl TryFrom<HeatEventXML> for HeatFinish {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        let _ = value.heat_id; // drop because we get inconsistent type from source

        if let Some(time) = value.time {
            if let Some(race_time) = value.runtime {
                Ok(Self {
                    application: value.application,
                    generated: value.generated.0,
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
impl TryFrom<HeatEventXML> for HeatIntermediate {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        let _ = value.heat_id; // drop because we get inconsistent type from source

        if let Some(time) = value.time {
            if let Some(race_time) = value.runtime {
                Ok(Self {
                    application: value.application,
                    generated: value.generated.0,
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
impl TryFrom<HeatEventXML> for HeatFalseStart {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        let _ = value.heat_id; // drop because we get inconsistent type from source

        if let Some(is_false_start) = value.is_false_start {
            if is_false_start {
                Ok(Self {
                    application: value.application,
                    generated: value.generated.0,
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
pub struct HeatStartListXML {
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

fn default_str_val() -> String {
    String::from("X")
}

fn default_str_empty() -> String {
    String::from("")
}

fn default_str_nation() -> String {
    String::from("GER")
}

#[derive(Deserialize, Clone)]
pub struct HeatCompetitorXML {
    #[serde(rename = "@Id")]
    id: String,
    #[serde(rename = "@Lane")]
    lane: u32,
    #[serde(rename = "@Bib")]
    bib: u32,
    #[serde(rename = "@Class")]
    #[serde(default = "default_str_val")]
    class: String,
    #[serde(rename = "@Lastname")]
    #[serde(default = "default_str_empty")]
    last_name: String,
    #[serde(rename = "@Firstname")]
    #[serde(default = "default_str_empty")]
    first_name: String,
    #[serde(rename = "@Nation")]
    #[serde(default = "default_str_nation")]
    nation: String,
    #[serde(rename = "@Club")]
    #[serde(default = "default_str_empty")]
    club: String,
    #[serde(rename = "@Gender")]
    #[serde(default = "default_str_val")]
    gender: String,
    #[serde(rename = "@Disqualification")]
    disqualification: Option<String>,
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
impl From<HeatStartListXML> for HeatStartList {
    fn from(value: HeatStartListXML) -> Self {
        let _ = value.heat_id; // drop because we get inconsistent type from source
        let _ = value.session_id;
        let _ = value.event_id;

        Self {
            name: value.name,
            id: value.id,
            nr: value.nr,
            session_nr: value.session_nr,
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
            disqualified: value
                .disqualification
                .map(|a| DisqualificationReason::parse_from_string(&a)),
        }
    }
}

#[derive(Deserialize)]
pub struct HeatWindXML {
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
    wind: Option<f32>,
    #[serde(rename = "@WindUnit")]
    wind_unit: String,
}
impl TryFrom<HeatWindXML> for HeatWind {
    fn try_from(value: HeatWindXML) -> Result<Self, Self::Error> {
        if value.wind_unit != "MetersPerSecond" {
            return Err("Can only parse wind that is in unit 'MetersPerSecond'".into());
        }

        let _ = value.heat_id; // drop because we get inconsistent type from source
        let _ = value.session_id;
        let _ = value.event_id;

        Ok(Self {
            id: value.id,
            application: value.application,
            generated: value.generated.0,
            version: value.version,
            wind: RaceWind::parse_from_f32(value.wind.ok_or(String::from(
                "Tried to parse a wind, but no wind field present",
            ))?),
        })
    }
    type Error = String;
}
impl From<HeatWindXML> for HeatWindMissing {
    fn from(value: HeatWindXML) -> Self {
        let _ = value.heat_id; // drop because we get inconsistent type from source
        let _ = value.session_id;
        let _ = value.event_id;

        Self {
            id: value.id,
            application: value.application,
            generated: value.generated.0,
            version: value.version,
        }
    }
}

#[derive(Deserialize)]
pub struct CompetitorEvaluatedXML {
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
    #[serde(default = "default_str_val")]
    class: String,
    #[serde(rename = "@Lastname")]
    #[serde(default = "default_str_empty")]
    last_name: String,
    #[serde(rename = "@Firstname")]
    #[serde(default = "default_str_empty")]
    first_name: String,
    #[serde(rename = "@Nation")]
    #[serde(default = "default_str_nation")]
    nation: String,
    #[serde(rename = "@Club")]
    #[serde(default = "default_str_empty")]
    club: String,
    #[serde(rename = "@Gender")]
    #[serde(default = "default_str_val")]
    gender: String,
    #[serde(rename = "@Disqualification")]
    disqualification: Option<String>,
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
                disqualified: value
                    .disqualification
                    .map(|a| DisqualificationReason::parse_from_string(&a)),
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

        let _ = value.heat_id; // drop because we get inconsistent type from source
        let _ = value.session_id;
        let _ = value.event_id;

        Ok(Self {
            id: value.id,
            application: value.application,
            generated: value.generated.0,
            version: value.version,
            competitor_result: competitor_result,
        })
    }
    type Error = String;
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
pub struct HeatResultXML {
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
    wind: Option<f32>,
    #[serde(rename = "@WindUnit")]
    wind_unit: Option<String>,
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
impl TryFrom<HeatResultXML> for HeatResult {
    fn try_from(value: HeatResultXML) -> Result<Self, Self::Error> {
        if let Some(wind_unit) = value.wind_unit {
            if wind_unit != "MetersPerSecond" {
                return Err("Can only parse wind that is in unit 'MetersPerSecond'".into());
            }
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

        let _ = value.heat_id; // drop because we get inconsistent type from source
        let _ = value.session_id;
        let _ = value.event_id;

        Ok(Self {
            id: value.id,
            wind: value.wind.map(|w| RaceWind::parse_from_f32(w)),
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
                            Err(e) => {
                                return Err(format!(
                                    "Must be start, could not parse: {}\n{}",
                                    e, decoded_string
                                ))
                            }
                        }
                    }
                } else if decoded_string.starts_with("<HeatFinish ") {
                    match HeatFinish::try_from(event) {
                        Ok(converted) => {
                            return Ok(InstructionFromCameraProgram::HeatFinish(converted))
                        }
                        Err(e) => {
                            return Err(format!(
                                "Must be finish, could not parse: {}\n{}",
                                e, decoded_string
                            ))
                        }
                    }
                } else {
                    match HeatIntermediate::try_from(event) {
                        Ok(converted) => {
                            return Ok(InstructionFromCameraProgram::HeatIntermediate(converted))
                        }
                        Err(e) => {
                            return Err(format!(
                                "Must be intermediate, could not parse: {}\n{}",
                                e, decoded_string
                            ))
                        }
                    }
                }
            }
            Err(e) => {
                return Err(format!(
                    "Heat start, finish or intermediate parse error: {}\n{}",
                    e.to_string(),
                    decoded_string
                ))
            }
        }
    }

    if decoded_string.starts_with("<HeatStartlist ") {
        match from_str::<HeatStartListXML>(&decoded_string) {
            Ok(dec) => return Ok(InstructionFromCameraProgram::HeatStartList(dec.into())),
            Err(e) => {
                return Err(format!(
                    "Heat start list parse error: {}\n{}",
                    e.to_string(),
                    decoded_string
                ))
            }
        }
    }

    if decoded_string.starts_with("<HeatWind ") {
        match from_str::<HeatWindXML>(&decoded_string) {
            Ok(dec) => {
                if dec.wind.is_none() {
                    // detached wind sensor or somethingth
                    return Ok(InstructionFromCameraProgram::HeatWindMissing(dec.into()));
                }
                match HeatWind::try_from(dec) {
                    Ok(dec) => return Ok(InstructionFromCameraProgram::HeatWind(dec)),
                    Err(e) => {
                        return Err(format!(
                            "Wind could not be decoded: {}\n{}",
                            e.to_string(),
                            decoded_string
                        ))
                    }
                }
            }
            Err(e) => {
                return Err(format!(
                    "Heat wind parse error: {}\n{}",
                    e.to_string(),
                    decoded_string
                ))
            }
        };
    }

    if decoded_string.starts_with("<CompetitorEvaluated ") {
        match from_str::<CompetitorEvaluatedXML>(&decoded_string) {
            Ok(dec) => match CompetitorEvaluated::try_from(dec) {
                Ok(dec) => return Ok(InstructionFromCameraProgram::CompetitorEvaluated(dec)),
                Err(e) => {
                    return Err(format!(
                        "Competitor Evaluation could not be decoded: {}\n{}",
                        e.to_string(),
                        decoded_string
                    ))
                }
            },
            Err(e) => {
                return Err(format!(
                    "Competitor evaluated parse error: {}\n{}",
                    e.to_string(),
                    decoded_string
                ))
            }
        };
    }

    if decoded_string.starts_with("<HeatResult ") {
        match from_str::<HeatResultXML>(&decoded_string) {
            Ok(dec) => match HeatResult::try_from(dec) {
                Ok(dec) => return Ok(InstructionFromCameraProgram::HeatResult(dec)),
                Err(e) => {
                    return Err(format!(
                        "Heat Result cloud not be decoded: {}\n{}",
                        e.to_string(),
                        decoded_string
                    ))
                }
            },
            Err(e) => {
                return Err(format!(
                    "Heat result evaluated parse error: {}\n{}",
                    e.to_string(),
                    decoded_string
                ))
            }
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
