use std::time::Duration;

use async_channel::{Receiver, RecvError, SendError, Sender};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio::time::{self, error::Elapsed};

use crate::{args::Args, hex::parse_race_time};

#[derive(Debug)]
pub enum IncomingInstruction {
    FromTimingClient(InstructionFromTimingClient),
    FromCameraProgram(InstructionFromCameraProgram),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RaceTime {
    pub hours: Option<u16>,
    pub minutes: Option<u16>,
    pub seconds: u16,
    pub tenths: u16,
    pub hundrets: Option<u16>,
    pub thousands: Option<u16>,
    pub ten_thousands: Option<u16>,
}
impl RaceTime {
    pub fn optimize_representation_for_display(&self) -> Self {
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

        Self {
            hours: hours_out,
            minutes: minutes_out,
            seconds: self.seconds,
            tenths: self.tenths,
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
            "{}{}{}.{}{}{}{}",
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
            format!("{}", self.tenths % 10),
            if let Some(hundrets) = self.hundrets {
                format!("{}", hundrets % 10)
            } else {
                String::from("")
            },
            if let Some(thousands) = self.thousands {
                format!("{}", thousands % 10)
            } else {
                String::from("")
            },
            if let Some(ten_thousands) = self.ten_thousands {
                format!("{}", ten_thousands % 10)
            } else {
                String::from("")
            },
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
                let fractional_part_in_ten_thousands: u32 = rt.tenths as u32 * 1000
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
pub enum InstructionFromCameraProgram {
    DayTime(DayTime),
    RaceTime(RaceTime),
    IntermediateTime(RaceTime),
    EndTime(RaceTime),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InstructionFromTimingClient {
    ClientInfo,
    Freetext(String),
    Advertisements,
    Clear,
    StartList,
    Timing,
    Meta,
    TimeText,
    Results,
    ResultsUpdate,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InstructionToTimingClient {
    SendBeforeFrameSetupInstruction,
    SendFrame, // TODO needs to store the frame data
}

#[derive(Clone)]
pub struct InstructionCommunicationChannel {
    args: Args,
    inbound_sender: Sender<IncomingInstruction>,
    inbound_receiver: Receiver<IncomingInstruction>,
    outbound_sender: Sender<InstructionToTimingClient>,
    outbound_receiver: Receiver<InstructionToTimingClient>,
}
impl InstructionCommunicationChannel {
    pub fn new(args: &Args) -> Self {
        let (is, ir) = async_channel::unbounded::<IncomingInstruction>();
        let (os, or) = async_channel::unbounded::<InstructionToTimingClient>();

        Self {
            args: args.clone(),
            inbound_sender: is,
            inbound_receiver: ir,
            outbound_sender: os,
            outbound_receiver: or,
        }
    }

    pub async fn take_in_command_from_timing_client(
        &self,
        inst: InstructionFromTimingClient,
    ) -> Result<(), SendError<IncomingInstruction>> {
        self.inbound_sender
            .send(IncomingInstruction::FromTimingClient(inst))
            .await
    }

    pub async fn take_in_command_from_camera_program(
        &self,
        inst: InstructionFromCameraProgram,
    ) -> Result<(), SendError<IncomingInstruction>> {
        self.inbound_sender
            .send(IncomingInstruction::FromCameraProgram(inst))
            .await
    }

    pub async fn wait_for_incomming_command(
        &self,
    ) -> Result<Result<IncomingInstruction, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.inbound_receiver.recv(),
        )
        .await
    }

    pub async fn send_out_command(
        &self,
        inst: InstructionToTimingClient,
    ) -> Result<(), SendError<InstructionToTimingClient>> {
        self.outbound_sender.send(inst).await
    }

    pub async fn wait_for_command_to_send(
        &self,
    ) -> Result<Result<InstructionToTimingClient, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.outbound_receiver.recv(),
        )
        .await
    }
}
