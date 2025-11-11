use std::time::Duration;

use async_channel::{Receiver, RecvError, Sender, TrySendError};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio::time::{self, error::Elapsed};

use crate::{
    args::{Args, MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS},
    hex::parse_race_time,
    interface::MessageFromServerToClient,
    xml_serial::{
        CompetitorEvaluated, HeatFalseStart, HeatFinish, HeatIntermediate, HeatResult, HeatStart,
        HeatStartList, HeatWind,
    },
};

pub enum IncomingInstruction {
    FromTimingClient(InstructionFromTimingClient),
    FromCameraProgram(InstructionFromCameraProgram),
}
impl Display for IncomingInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                IncomingInstruction::FromTimingClient(tci) => format!("FromTimingClient: {}", tci),
                IncomingInstruction::FromCameraProgram(cpi) =>
                    format!("FromCameraProgram: {:?}", cpi),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
pub struct RaceWind {
    /// - is gegenwind, + is rückenwind aka back wind
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

#[derive(Debug, Serialize, Deserialize)]
pub enum InstructionFromCameraProgram {
    ZeroTime,
    DayTime(DayTime),
    RaceTime(RaceTime),
    IntermediateTime(RaceTime),
    EndTime(RaceTime),
    HeatStart(HeatStart),
    HeatFalseStart(HeatFalseStart),
    HeatStartList(HeatStartList),
    HeatWind(HeatWind),
    HeatIntermediate(HeatIntermediate),
    HeatFinish(HeatFinish),
    CompetitorEvaluated(CompetitorEvaluated),
    HeatResult(HeatResult),
}

#[derive(Serialize, Deserialize)]
pub enum InstructionFromTimingClient {
    ClientInfo,
    Freetext(String),
    Advertisements,
    Clear,
    StartList,
    Timing,
    SetProperty,
    Results,
    ResultsUpdate,
    ServerInfo,
    SendFrame(Vec<u8>),
}
impl Display for InstructionFromTimingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InstructionFromTimingClient::ClientInfo => String::from("ClientInfo"),
                InstructionFromTimingClient::Freetext(text) => format!("Freetext: {}", text),
                InstructionFromTimingClient::Advertisements => String::from("Advertisements"),
                InstructionFromTimingClient::Clear => String::from("Clear"),
                InstructionFromTimingClient::StartList => String::from("StartList"),
                InstructionFromTimingClient::Timing => String::from("Timing"),
                InstructionFromTimingClient::SetProperty => String::from("SetProperty"),
                InstructionFromTimingClient::Results => String::from("Results"),
                InstructionFromTimingClient::ResultsUpdate => String::from("ResultsUpdate"),
                InstructionFromTimingClient::ServerInfo => String::from("ServerInfo"),
                InstructionFromTimingClient::SendFrame(_) => String::from("SendFrame"),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InstructionToTimingClient {
    SendServerInfo,
    SendFrame(Vec<u8>), // stores the frame data
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
        let (is, ir) = async_channel::bounded::<IncomingInstruction>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        let (os, or) = async_channel::bounded::<InstructionToTimingClient>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );

        Self {
            args: args.clone(),
            inbound_sender: is,
            inbound_receiver: ir,
            outbound_sender: os,
            outbound_receiver: or,
        }
    }

    pub fn take_in_command_from_timing_client(
        &self,
        inst: InstructionFromTimingClient,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromTimingClient(inst))
        {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Internal communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Internal communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub fn take_in_command_from_camera_program(
        &self,
        inst: InstructionFromCameraProgram,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromCameraProgram(inst))
        {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Internal communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Internal communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
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

    pub fn send_out_command(&self, inst: InstructionToTimingClient) -> Result<(), String> {
        match self.outbound_sender.try_send(inst) {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Internal communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Internal communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
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

#[derive(Clone)]
pub struct ClientCommunicationChannelOutbound {
    args: Args,
    sender: Sender<MessageFromServerToClient>,
    receiver: Receiver<MessageFromServerToClient>,
}
impl ClientCommunicationChannelOutbound {
    pub fn new(args: &Args) -> Self {
        let (s, r) = async_channel::bounded::<MessageFromServerToClient>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );

        Self {
            args: args.clone(),
            sender: s,
            receiver: r,
        }
    }

    pub fn send_away(&self, inst: MessageFromServerToClient) -> Result<(), String> {
        match self.sender.try_send(inst) {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Internal communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Internal communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub async fn wait_for_message_to_send(
        &self,
    ) -> Result<Result<MessageFromServerToClient, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.receiver.recv(),
        )
        .await
    }
}
