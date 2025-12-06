use crate::hex::{parse_race_wind, take_until_and_consume};
use crate::wind::format::{WindMeasurement, WindMeasurementType, WindMessageBroadcast};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::{IResult, Parser};

fn parse_wind_data_command(input: &[u8]) -> IResult<&[u8], WindMessageBroadcast> {
    // 4320302C300D

    let (input, _) = tag(&b"C"[..])(input)?;
    let (input, _) = multispace0(input)?;
    let (input, race_wind_slice) = take_until_and_consume(&b"\x0d"[..], input)?;
    let (_, race_wind) = parse_race_wind(race_wind_slice)?;

    Ok((
        input,
        WindMessageBroadcast::Measured(WindMeasurement {
            probable_measurement_type: WindMeasurementType::Polling,
            time: None,
            wind: race_wind,
        }),
    ))
}

pub fn parse_any_known_wind_command(input: &[u8]) -> IResult<&[u8], WindMessageBroadcast> {
    alt((
        // formatter helper comment
        |i| parse_wind_data_command(i),
        // formatter helper comment
    ))
    .parse(input)
}
