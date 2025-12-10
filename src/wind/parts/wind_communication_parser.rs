use crate::hex::{parse_race_wind, take_until_and_consume, NomErr, NomError, NomErrorKind};
use crate::wind::format::{
    StartedWindMeasurement, WindMeasurement, WindMeasurementType, WindMessageBroadcast,
};
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::multispace0;
use nom::combinator::map_opt;
use nom::{IResult, Parser};

fn parse_wind_data_command(input: &[u8]) -> IResult<&[u8], Option<WindMessageBroadcast>> {
    // 43 20 30 2c 30 0d <- zero
    // 43 20 31 2c 36 0d <- backwind = 20 space
    // 43 2d 30 2c 30 0d <- headwind = 2d -
    // !! the first byte is 52, if it is a measurement result

    let (input, type_byte) = take(1usize)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, race_wind_slice) = take_until_and_consume(&b"\x0d"[..], input)?;
    let (_, race_wind) = parse_race_wind(race_wind_slice)?;

    let measurement_type = match type_byte[0] {
        0x43 => WindMeasurementType::Polling,
        0x52 => WindMeasurementType::UnidentifiedMeasurement,
        _ => return Err(NomErr(NomError::new(input, NomErrorKind::Tag))),
    };

    Ok((
        input,
        Some(WindMessageBroadcast::Measured(WindMeasurement {
            probable_measurement_type: measurement_type,
            time: None,
            wind: race_wind,
        })),
    ))
}

fn parse_wind_measurement_command(input: &[u8]) -> IResult<&[u8], Option<WindMessageBroadcast>> {
    // 53 50 32 XX 5c 72 0d

    // XX means the following:
    // 4e: N -> Normal polling -> WindMeasurementType::Polling
    // 57: W -> 5s Weit -> WindMeasurementType::Jump5s
    // 54: T -> 8s ??? -> WindMeasurementType::Other8s
    // 4c: L -> 10s Lauf -> WindMeasurementType::Race10s
    // 5a: Z -> 12s ??? -> WindMeasurementType::Other12s
    // 48: H -> 13s Hürden -> WindMeasurementType::Race13s

    let (input, (_, ms_type, _)) = (
        tag("SP2"),
        map_opt(nom::bytes::complete::take(1usize), |b: &[u8]| {
            match b[0] {
                0x4E => Some(WindMeasurementType::Polling),  // N
                0x57 => Some(WindMeasurementType::Jump5s),   // W
                0x54 => Some(WindMeasurementType::Other8s),  // T
                0x4C => Some(WindMeasurementType::Race10s),  // L
                0x5A => Some(WindMeasurementType::Other12s), // Z
                0x48 => Some(WindMeasurementType::Race13s),  // H
                _ => None,
            }
        }),
        tag("\x5C\x72\x0D"), // '\' 'r' CR
    )
        .parse(input)?;

    Ok((
        input,
        Some(WindMessageBroadcast::Started(StartedWindMeasurement {
            ms_type,
            time: None,
        })),
    ))
}

fn parse_ack_commant(input: &[u8]) -> IResult<&[u8], Option<WindMessageBroadcast>> {
    // 4f -> ??? probably some kind of ack
    let (input, _) = tag(&b"\x4f"[..])(input)?;

    Ok((input, None))
}

pub fn parse_any_known_wind_command(input: &[u8]) -> IResult<&[u8], Option<WindMessageBroadcast>> {
    alt((
        // formatter helper comment
        |i| parse_wind_data_command(i),
        |i| parse_wind_measurement_command(i),
        |i| parse_ack_commant(i),
        // formatter helper comment
    ))
    .parse(input)
}
