use crate::hex::parse_race_time;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RaceTime {
    pub hours: Option<u16>,
    pub minutes: Option<u16>,
    pub seconds: u16,
    pub tenths: Option<u16>,
    pub hundrets: Option<u16>,
    pub thousands: Option<u16>,
    pub ten_thousands: Option<u16>,
}
impl RaceTime {
    pub fn optimize_representation_for_display(
        &self,
        force_number_of_decimal_places: Option<i8>,
    ) -> Self {
        let mut hours_out = self.hours;
        if let Some(hours_out_val) = hours_out {
            if hours_out_val == 0 {
                hours_out = None;
            }
        }

        let mut minutes_out = self.minutes;
        if hours_out.is_none() {
            if let Some(minutes_out_val) = minutes_out {
                if minutes_out_val == 0 {
                    minutes_out = None;
                }
            }
        }

        let mut tenths_out = self.tenths;
        let mut hundrets_out = self.hundrets;
        let mut thousands_out = self.thousands;
        let mut ten_thousands_out = self.ten_thousands;
        if hours_out.is_some() {
            hundrets_out = None;
        }
        if minutes_out.is_some() {
            thousands_out = None;
            ten_thousands_out = None;
        }

        if let Some(force_number_of_decimal_places) = force_number_of_decimal_places {
            match force_number_of_decimal_places {
                -1 => {
                    // way to ignore setting if it is set (== None)
                }
                0 => {
                    tenths_out = None;
                    hundrets_out = None;
                    thousands_out = None;
                    ten_thousands_out = None;
                }
                1 => {
                    tenths_out = Some(self.tenths.unwrap_or(0));
                    hundrets_out = None;
                    thousands_out = None;
                    ten_thousands_out = None;
                }
                2 => {
                    tenths_out = Some(self.tenths.unwrap_or(0));
                    hundrets_out = Some(self.hundrets.unwrap_or(0));
                    thousands_out = None;
                    ten_thousands_out = None;
                }
                3 => {
                    tenths_out = Some(self.tenths.unwrap_or(0));
                    hundrets_out = Some(self.hundrets.unwrap_or(0));
                    thousands_out = Some(self.thousands.unwrap_or(0));
                    ten_thousands_out = None;
                }
                4 => {
                    tenths_out = Some(self.tenths.unwrap_or(0));
                    hundrets_out = Some(self.hundrets.unwrap_or(0));
                    thousands_out = Some(self.thousands.unwrap_or(0));
                    ten_thousands_out = Some(self.ten_thousands.unwrap_or(0));
                }
                _ => {}
            }
        }

        Self {
            hours: hours_out,
            minutes: minutes_out,
            seconds: self.seconds,
            tenths: tenths_out,
            hundrets: hundrets_out,
            thousands: thousands_out,
            ten_thousands: ten_thousands_out,
        }
    }

    pub fn parse_from_string(input: &str) -> Result<Self, String> {
        match parse_race_time(&input.as_bytes()) {
            Ok((_, rt)) => Ok(rt),
            Err(e) => Err(e.to_string()),
        }
    }
}
impl Display for RaceTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}{}{}{}",
            if let Some(hours) = self.hours {
                format!("{}:", hours)
            } else {
                String::from("")
            },
            if let Some(minutes) = self.minutes {
                if let Some(_) = self.hours {
                    format!("{:02}:", minutes)
                } else {
                    format!("{}:", minutes)
                }
            } else {
                String::from("")
            },
            if self.minutes.is_some() || self.hours.is_some() {
                format!("{:02}", self.seconds)
            } else {
                format!("{}", self.seconds)
            },
            if self.tenths.is_some()
                || self.hundrets.is_some()
                || self.thousands.is_some()
                || self.ten_thousands.is_some()
            {
                "."
            } else {
                ""
            },
            if let Some(tenths) = self.tenths {
                format!("{}", tenths % 10)
            } else {
                if self.hundrets.is_some()
                    || self.thousands.is_some()
                    || self.ten_thousands.is_some()
                {
                    String::from("0")
                } else {
                    String::from("")
                }
            },
            if let Some(hundrets) = self.hundrets {
                format!("{}", hundrets % 10)
            } else {
                if self.thousands.is_some() || self.ten_thousands.is_some() {
                    String::from("0")
                } else {
                    String::from("")
                }
            },
            if let Some(thousands) = self.thousands {
                format!("{}", thousands % 10)
            } else {
                if self.ten_thousands.is_some() {
                    String::from("0")
                } else {
                    String::from("")
                }
            },
            if let Some(ten_thousands) = self.ten_thousands {
                format!("{}", ten_thousands % 10)
            } else {
                String::from("")
            },
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DayTime {
    pub hours: u16,
    pub minutes: u16,
    pub seconds: u16,
    pub fractional_part_in_ten_thousands: Option<u32>,
}
impl DayTime {
    pub fn to_exact_string(&self) -> String {
        format!(
            "{}.{:04}",
            self.to_string(),
            self.fractional_part_in_ten_thousands.unwrap_or(0) % 10000
        )
    }

    pub fn parse_from_string(input: &str) -> Result<Self, String> {
        match parse_race_time(&input.as_bytes()) {
            Ok((_, rt)) => {
                if rt.hours.is_none() || rt.minutes.is_none() {
                    return Err(String::from("DayTime needs hours and minutes!"));
                }

                let hours = rt.hours.unwrap_or(0);
                let minutes = rt.minutes.unwrap_or(0);
                let fractional_part_in_ten_thousands: u32 = rt.tenths.unwrap_or(0) as u32 * 1000
                    + rt.hundrets.unwrap_or(0) as u32 * 100
                    + rt.thousands.unwrap_or(0) as u32 * 10
                    + rt.ten_thousands.unwrap_or(0) as u32;

                Ok(DayTime {
                    hours,
                    minutes,
                    seconds: rt.seconds,
                    fractional_part_in_ten_thousands: Some(fractional_part_in_ten_thousands),
                })
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
impl Display for DayTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.hours, self.minutes, self.seconds
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RaceWind {
    /// - is gegenwind aka head wind, + is rückenwind aka back wind
    pub back_wind: bool,
    pub whole_number_part: u16,
    pub fraction_part: u8, // 0-9
}
impl RaceWind {
    pub fn parse_from_f32(input: f32) -> Self {
        let mut is_back_wind = true;
        if input < 0.0 {
            // 0.0 should be +
            is_back_wind = false;
        }

        let positive = input.abs();
        let whole_part = positive.floor().clamp(0.0, u8::MAX as f32);
        let fraction_part = ((((positive - whole_part) * 10.0).floor() as u32) % 10) as u8;

        Self {
            back_wind: is_back_wind,
            whole_number_part: whole_part as u16,
            fraction_part: fraction_part,
        }
    }
}
impl Display for RaceWind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}.{}",
            if self.back_wind { "+" } else { "-" },
            self.whole_number_part,
            self.fraction_part
        )
    }
}
