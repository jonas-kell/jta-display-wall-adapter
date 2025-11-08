use crate::hex::{parse_race_time, parse_two_digits, take_until_and_consume};
use chrono::{NaiveDateTime, NaiveTime};
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

const STARTTIME_FORMAT: &str = "%H:%M:%S%.f";

#[derive(Clone)]
struct HeatStartTime(NaiveTime);

impl<'de> Deserialize<'de> for HeatStartTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveTime::parse_from_str(&s, STARTTIME_FORMAT)
            .map(HeatStartTime)
            .map_err(serde::de::Error::custom)
    }
}

const PLANNED_STARTTIME_FORMAT: &str = "%H:%M:%S";

struct HeatPlannedStartTime(NaiveTime);

impl<'de> Deserialize<'de> for HeatPlannedStartTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveTime::parse_from_str(&s, PLANNED_STARTTIME_FORMAT)
            .map(HeatPlannedStartTime)
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
    time: Option<HeatStartTime>,
    #[serde(rename = "@Runtime")]
    run_time: Option<HeatRaceTime>,
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
    pub time: NaiveTime,
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
    pub time: NaiveTime,
    pub race_time: RaceTime,
}
impl TryFrom<HeatEventXML> for HeatFinish {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        if let Some(time) = value.time {
            if let Some(race_time) = value.run_time {
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
    pub time: NaiveTime,
    pub intermediate_time_at: RaceTime,
}
impl TryFrom<HeatEventXML> for HeatIntermediate {
    fn try_from(value: HeatEventXML) -> Result<Self, Self::Error> {
        if let Some(time) = value.time {
            if let Some(race_time) = value.run_time {
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
    scheduled_start_time: HeatPlannedStartTime,
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
    pub scheduled_start_time: NaiveTime,
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
            return Err("Can only parse wind that is ins unit 'MetersPerSecond'".into());
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

// https://docs.rs/quick-xml/latest/quick_xml/de/index.html
fn decode_single_xml(packet: &[u8]) -> Result<InstructionFromCameraProgram, String> {
    let decoded_string: String = String::from_utf8_lossy(packet).to_string();

    // the XML parser does not care for the root element name... Needs to be checked manually
    if decoded_string.starts_with("<HeatStart ")
        || decoded_string.starts_with("<HeatFinish ")
        || decoded_string.starts_with("<HeatIntermediate ")
    {
        if let Ok(event) = from_str::<HeatEventXML>(&decoded_string) {
            if decoded_string.starts_with("<HeatStart ") {
                if let Ok(converted) = HeatFalseStart::try_from(event.clone()) {
                    return Ok(InstructionFromCameraProgram::HeatFalseStart(converted));
                } else if let Ok(converted) = HeatStart::try_from(event) {
                    return Ok(InstructionFromCameraProgram::HeatStart(converted));
                } else {
                    return Err("Must be start, but required property missing".into());
                }
            } else if decoded_string.starts_with("<HeatFinish ") {
                if let Ok(converted) = HeatFinish::try_from(event) {
                    return Ok(InstructionFromCameraProgram::HeatFinish(converted));
                } else {
                    return Err("Must be finish, but required property missing".into());
                }
            } else {
                if let Ok(converted) = HeatIntermediate::try_from(event) {
                    return Ok(InstructionFromCameraProgram::HeatIntermediate(converted));
                } else {
                    return Err("Must be intermediated, but required property missing".into());
                }
            }
        }

        if let Err(asd) = from_str::<HeatEventXML>(&decoded_string) {
            error!("Hello; {}", asd);
        }
    }

    if decoded_string.starts_with("<HeatStartlist ") {
        if let Ok(dec) = from_str::<HeatStartListXML>(&decoded_string) {
            return Ok(InstructionFromCameraProgram::HeatStartList(dec.into()));
        }
    }

    if decoded_string.starts_with("<HeatWind ") {
        match from_str::<HeatWindXML>(&decoded_string) {
            Ok(dec) => match HeatWind::try_from(dec) {
                Ok(dec) => return Ok(InstructionFromCameraProgram::HeatWind(dec)),
                Err(err) => return Err(format!("Wind could not be decoded: {}", err.to_string())),
            },
            Err(_) => (), // logging was here in the beginning. No not because of fallthrough
        };
    }

    debug!("XML message:\n{}", decoded_string);

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
