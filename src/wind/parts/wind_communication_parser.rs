use crate::hex::{parse_race_time, take_until_and_consume};
use crate::times::RaceWind;
use crate::wind::format::{WindMeasurement, WindMeasurementType, WindMessageBroadcast};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::{IResult, Parser};

fn parse_wind_data_command(input: &[u8]) -> IResult<&[u8], WindMessageBroadcast> {
    let (input, _) = tag(&b"\x20\x20\x20\x42"[..])(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _rt) = parse_race_time(input)?;
    let (input, _) = take_until_and_consume(&b"\x0D"[..], input)?;

    Ok((
        input,
        WindMessageBroadcast::Measured(WindMeasurement {
            probable_measurement_type: WindMeasurementType::Polling,
            time: None,
            wind: RaceWind {
                back_wind: true,
                whole_number_part: 11,
                fraction_part: 0,
            },
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
