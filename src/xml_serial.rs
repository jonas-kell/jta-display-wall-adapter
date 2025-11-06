use crate::hex::{parse_race_time, parse_two_digits, take_until_and_consume};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::{IResult, Parser};

use crate::args::Args;
use crate::hex::hex_log_bytes;
use crate::instructions::DayTime;
use crate::instructions::InstructionFromCameraProgram;

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

fn decode_single_xml(packet: &[u8]) -> Result<InstructionFromCameraProgram, String> {
    let decoded: String = String::from_utf8_lossy(packet).to_string();

    debug!("XML message:\n{}", decoded);

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

    // 202020202020202031363A31323A32382020202020200D4220202020202020202020202020203220202033202020

    let dt = DayTime {
        hours,
        minutes,
        seconds,
        fractional_part_in_ten_thousands: None,
    };

    Ok((input, InstructionFromCameraProgram::DayTime(dt)))
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
        |i| parse_clock_command(i),
        |i| parse_intermediate_time_command(i),
        |i| parse_end_time_command(i),
        |i| parse_time_command(i),
    ))
    .parse(input)
}
