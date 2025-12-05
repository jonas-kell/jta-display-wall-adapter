use crate::hex::{parse_race_time, take_until_and_consume};
use crate::wind::format::WindMessageBroadcast;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::{IResult, Parser};

pub fn decode_single_usb_dump(packet: &[u8]) -> Vec<Result<WindMessageBroadcast, String>> {
    // we expect the sniffer to output valid formatted string data -> can lossy convert here.
    let decoded = String::from_utf8_lossy(packet);

    let mut res = Vec::new();

    for decoded_line in decoded.lines() {
        match parse_any_known_usb_command(decoded_line.as_bytes()) {
            Err(e) => error!("Nom parser Error: {}", e.to_string()),
            Ok((_, opt_comm)) => {
                match opt_comm {
                    None => {
                        // Not a message to pass on
                        continue;
                    }
                    Some(command) => {
                        res.push(Ok(command));
                    }
                }
            }
        }

        res.push(Err(format!(
            "No wind communication could be parsed from the following USB data frame line: {}",
            decoded_line
        )));
    }

    res
}

fn parse_usb_data_line(input: &[u8]) -> IResult<&[u8], Option<WindMessageBroadcast>> {
    let (input, _) = tag(&b"\x20\x20\x20\x42"[..])(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _rt) = parse_race_time(input)?;
    let (input, _) = take_until_and_consume(&b"\x0D"[..], input)?;

    // TODO

    Ok((input, None))
}

fn throwaway_parse_line(input: &[u8]) -> IResult<&[u8], Option<WindMessageBroadcast>> {
    Ok((input, None))
}

fn parse_any_known_usb_command(input: &[u8]) -> IResult<&[u8], Option<WindMessageBroadcast>> {
    alt((
        // formatter helper comment
        |i| parse_usb_data_line(i),
        |i| throwaway_parse_line(i),
        // formatter helper comment
    ))
    .parse(input)
}
