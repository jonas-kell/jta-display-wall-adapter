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
        let mut real_force_number_of_decimal_places: i8 = -1;
        if let Some(force_number_of_decimal_places) = force_number_of_decimal_places {
            real_force_number_of_decimal_places = force_number_of_decimal_places;
        }
        if real_force_number_of_decimal_places < 0 {
            // way to ignore setting if it is set (== None)

            // set as high as there is precision given
            if self.ten_thousands.is_some() {
                real_force_number_of_decimal_places = 4;
            }
            if self.thousands.is_some() {
                real_force_number_of_decimal_places = 3;
            }
            if self.hundrets.is_some() {
                real_force_number_of_decimal_places = 2;
            }
            if self.tenths.is_some() {
                real_force_number_of_decimal_places = 1;
            }

            // do not over-extend if precicion is not given
            if self.ten_thousands.is_none() {
                real_force_number_of_decimal_places = 3;
            }
            if self.thousands.is_none() && self.ten_thousands.is_none() {
                real_force_number_of_decimal_places = 2;
            }
            if self.hundrets.is_none() && self.thousands.is_none() && self.ten_thousands.is_none() {
                real_force_number_of_decimal_places = 1;
            }
            if self.tenths.is_none()
                && self.hundrets.is_none()
                && self.thousands.is_none()
                && self.ten_thousands.is_none()
            {
                real_force_number_of_decimal_places = 0;
            }

            // -> automatic hiding
            if self.minutes.is_some() {
                real_force_number_of_decimal_places = 2;
            }
            if self.hours.is_some() {
                real_force_number_of_decimal_places = 1;
            }
        }
        if real_force_number_of_decimal_places < 0 {
            // sanity check.
            real_force_number_of_decimal_places = 0;
        }

        let hours_calc = self.hours.unwrap_or(0);
        let minutes_calc = self.minutes.unwrap_or(0);
        let seconds_calc = self.seconds;
        let tenths_calc = self.tenths.unwrap_or(0);
        let hundrets_calc = self.hundrets.unwrap_or(0);
        let thousands_calc = self.thousands.unwrap_or(0);
        let ten_thousands_calc = self.ten_thousands.unwrap_or(0);

        // how many 1/10000th fractions per unit
        const TEN_THOUSAND: u64 = 1;
        const THOUSAND: u64 = 10;
        const HUNDRED: u64 = 100;
        const TENTH: u64 = 1000;
        const SECOND: u64 = 10000;
        const MINUTE: u64 = 60 * 10000;
        const HOUR: u64 = 60 * 60 * 10000;

        let mut accumulated_time_in_ten_thousands: u64 = ten_thousands_calc as u64 * TEN_THOUSAND
            + thousands_calc as u64 * THOUSAND
            + hundrets_calc as u64 * HUNDRED
            + tenths_calc as u64 * TENTH
            + seconds_calc as u64 * SECOND
            + minutes_calc as u64 * MINUTE
            + hours_calc as u64 * HOUR;

        // ----- ROUNDING -----

        // digits: 0 = seconds only, 1 = +tenths, 2 = +hundreds, 3 = +thousands, 4+ = +ten_thousands
        let rounding_unit = match real_force_number_of_decimal_places {
            0 => SECOND,
            1 => TENTH,
            2 => HUNDRED,
            3 => THOUSAND,
            _ => TEN_THOUSAND,
        };

        let remainder = accumulated_time_in_ten_thousands % rounding_unit;

        if remainder * 2 >= rounding_unit {
            // round upward
            accumulated_time_in_ten_thousands += rounding_unit - remainder;
        } else {
            // round downward
            accumulated_time_in_ten_thousands -= remainder;
        }

        // ----- DECOMPOSE -----

        let hours_calc = (accumulated_time_in_ten_thousands / HOUR) as u16;
        accumulated_time_in_ten_thousands %= HOUR;
        let minutes_calc = (accumulated_time_in_ten_thousands / MINUTE) as u16;
        accumulated_time_in_ten_thousands %= MINUTE;
        let seconds_calc = (accumulated_time_in_ten_thousands / SECOND) as u16;
        accumulated_time_in_ten_thousands %= SECOND;
        let tenths_calc = (accumulated_time_in_ten_thousands / TENTH) as u16;
        accumulated_time_in_ten_thousands %= TENTH;
        let hundrets_calc = (accumulated_time_in_ten_thousands / HUNDRED) as u16;
        accumulated_time_in_ten_thousands %= HUNDRED;
        let thousands_calc = (accumulated_time_in_ten_thousands / THOUSAND) as u16;
        accumulated_time_in_ten_thousands %= THOUSAND;
        let ten_thousands_calc = accumulated_time_in_ten_thousands as u16;

        // ----- HIDE LEADING ZEROS -----
        let mut hours_out = Some(hours_calc);
        if hours_calc == 0 {
            hours_out = None;
        }

        let mut minutes_out = Some(minutes_calc);
        if hours_out.is_none() {
            if let Some(minutes_out_val) = minutes_out {
                if minutes_out_val == 0 {
                    minutes_out = None;
                }
            }
        }

        // ----- TRIM DECIMAL DIGITS  -----

        let tenths_opt = if real_force_number_of_decimal_places >= 1 {
            Some(tenths_calc)
        } else {
            None
        };
        let hundrets_opt = if real_force_number_of_decimal_places >= 2 {
            Some(hundrets_calc)
        } else {
            None
        };
        let thousands_opt = if real_force_number_of_decimal_places >= 3 {
            Some(thousands_calc)
        } else {
            None
        };
        let ten_thousands_opt = if real_force_number_of_decimal_places >= 4 {
            Some(ten_thousands_calc)
        } else {
            None
        };

        // ----- FINAL STRUCT -----

        Self {
            hours: hours_out,
            minutes: minutes_out,
            seconds: seconds_calc,
            tenths: tenths_opt,
            hundrets: hundrets_opt,
            thousands: thousands_opt,
            ten_thousands: ten_thousands_opt,
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
                    format!("{:02}:", minutes % 60)
                } else {
                    format!("{}:", minutes)
                }
            } else {
                if self.hours.is_some() {
                    String::from("00:")
                } else {
                    String::from("")
                }
            },
            if self.minutes.is_some() || self.hours.is_some() {
                format!("{:02}", self.seconds % 60)
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
