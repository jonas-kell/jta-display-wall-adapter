use crate::times::RaceTime;
use nom::bytes::complete::{take_while, take_while1};
use nom::combinator::{opt, peek};
pub use nom::error::{Error as NomError, ErrorKind as NomErrorKind};
use nom::sequence::preceded;
pub use nom::Err::{Error as NomErr, Failure as NomFailure};
use nom::Parser;
use nom::{bytes::complete::tag, bytes::complete::take_until, sequence::terminated, IResult};
use nom::{bytes::complete::take_while_m_n, combinator::map_res};

pub fn parse_two_digits<'a>(input: &'a [u8]) -> IResult<&'a [u8], u16> {
    map_res(
        take_while_m_n(2, 2, |c: u8| c.is_ascii_digit()),
        |bytes: &[u8]| {
            // Convert &[u8] → &str safely, then parse
            std::str::from_utf8(bytes)
                .map_err(|_| "invalid utf8")
                .and_then(|s| s.parse::<u16>().map_err(|_| "invalid number"))
        },
    )
    .parse(input)
}

pub fn take_until_and_consume<'a>(pattern: &[u8], input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    terminated(take_until(pattern), tag(pattern)).parse(input)
}

pub fn parse_u16(input: &[u8]) -> IResult<&[u8], u16> {
    map_res(take_while1(|c: u8| c.is_ascii_digit()), |bytes: &[u8]| {
        std::str::from_utf8(bytes)
            .map_err(|_| "invalid utf8")
            .and_then(|s| s.parse::<u16>().map_err(|_| "invalid number"))
    })
    .parse(input)
}

pub fn parse_race_time(input: &[u8]) -> IResult<&[u8], RaceTime> {
    // Detect how many colons exist

    let (_, time_slice) = peek(take_while(|b: u8| {
        b.is_ascii_digit() || b == b':' || b == b'.' || b == b','
    }))
    .parse(input)?;
    let colon_count = time_slice.iter().filter(|&&b| b == b':').count();

    let (input, (hours, minutes, seconds)) = match colon_count {
        2 => {
            // hours:minutes:seconds
            let (input, (h, _, m, _, s)) =
                (parse_u16, tag(":"), parse_u16, tag(":"), parse_u16).parse(input)?;
            (input, (Some(h), Some(m), s))
        }
        1 => {
            // minutes:seconds
            let (input, (m, _, s)) = (parse_u16, tag(":"), parse_u16).parse(input)?;
            (input, (None, Some(m), s))
        }
        0 => {
            // seconds only
            let (input, s) = parse_u16(input)?;
            (input, (None, None, s))
        }
        _ => return Err(NomErr(NomError::new(input, NomErrorKind::Count))),
    };

    // Optional fractional part
    let (input, fraction) =
        opt(preceded(tag("."), take_while1(|c: u8| c.is_ascii_digit()))).parse(input)?;

    let (tenths, hundrets, thousands, ten_thousands) = match fraction {
        Some(frac) => {
            let digits: Vec<u16> = frac.iter().take(4).map(|&b| (b - b'0') as u16).collect();
            (
                *digits.get(0).unwrap_or(&0),
                digits.get(1).copied(),
                digits.get(2).copied(),
                digits.get(3).copied(),
            )
        }
        None => (0, None, None, None),
    };

    Ok((
        input,
        RaceTime {
            hours,
            minutes,
            seconds,
            tenths: Some(tenths), // in all parsed representations they are there, but we may choose to omit for display pusposes
            hundrets,
            thousands,
            ten_thousands,
        },
    ))
}

// https://hexed.it/
pub fn get_hex_repr(buf: &[u8]) -> String {
    format!(
        // "\\x{}",
        "{}",
        buf.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            // .join("\\x")
            .join("")
    )
}

pub fn hex_log_bytes(buf: &[u8]) {
    trace!("({} bytes) Hex: \n{}", buf.len(), get_hex_repr(buf),);
}
