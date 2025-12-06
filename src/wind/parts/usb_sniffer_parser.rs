use crate::hex::{
    byte_parser, get_hex_repr, take_until_and_consume, NomError, NomErrorKind, NomFailure,
};
use crate::wind::format::WindMessageBroadcast;
use crate::wind::parts::wind_communication_parser::parse_any_known_wind_command;
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::multi::count;
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
                    }
                    Some(command) => {
                        res.push(Ok(command));
                    }
                }
                continue;
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
    //   6 : DATA1 (6): 43 20 30 2c 30 0d

    let (input, _) = take_until_and_consume(&b"\x44\x41\x54\x41"[..], input)?; // -> DATA
    let (input, _) = take(1usize)(input)?; // take the 0/1 -> that is realtively irrelevant for us.
    let (input, _) = tag(&b" ("[..])(input)?;
    let (input, num_as_bytes) = take_until_and_consume(&b"):"[..], input)?;

    let num_as_str = str::from_utf8(num_as_bytes)
        .map_err(|_| NomFailure(NomError::new(input, NomErrorKind::Alpha)))?;
    let num_bytes = usize::from_str_radix(num_as_str, 10)
        .map_err(|_| NomFailure(NomError::new(input, NomErrorKind::Digit)))?;

    let (input, parsed_bytes) = count(byte_parser(), num_bytes).parse(input)?;

    let (_, mes) = match parse_any_known_wind_command(&parsed_bytes) {
        Err(e) => {
            let kind = match e {
                NomFailure(e) | nom::Err::Error(e) => {
                    error!(
                        "Internal nom parsing error (that can not be propagated out): at <{}>: {:?}",
                        get_hex_repr(e.input),
                        e.code
                    );

                    e.code
                }
                nom::Err::Incomplete(_) => NomErrorKind::Complete, // should not happen? Do not entirely know what this is for
            };

            // here, we know it is a data line, but could not parse the inner content.
            // This will also make everything fail for the parsing of this line and not trigger throwaway_parse_line wildcard
            Err(NomFailure(NomError::new(input, kind)))
        }
        Ok(a) => Ok(a),
    }?;

    Ok((input, Some(mes)))
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
