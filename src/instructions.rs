use crate::{
    args::{Args, MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS},
    interface::MessageFromServerToClient,
    server::xml_types::{
        CompetitorEvaluated, HeatFalseStart, HeatFinish, HeatIntermediate, HeatResult, HeatStart,
        HeatStartList, HeatWind,
    },
    times::{DayTime, RaceTime},
};
use async_channel::{Receiver, RecvError, Sender, TrySendError};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::time::Duration;
use tokio::time::{self, error::Elapsed};

pub enum IncomingInstruction {
    FromTimingProgram(InstructionFromTimingProgram),
    FromCameraProgram(InstructionFromCameraProgram),
}
impl Display for IncomingInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                IncomingInstruction::FromTimingProgram(tci) =>
                    format!("FromTimingProgram: {}", tci),
                IncomingInstruction::FromCameraProgram(cpi) =>
                    format!("FromCameraProgram: {:?}", cpi),
            }
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
    HeatWindMissing,
    HeatIntermediate(HeatIntermediate),
    HeatFinish(HeatFinish),
    CompetitorEvaluated(CompetitorEvaluated),
    HeatResult(HeatResult),
}

#[derive(Serialize, Deserialize)]
pub enum InstructionFromTimingProgram {
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
impl Display for InstructionFromTimingProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InstructionFromTimingProgram::ClientInfo => String::from("ClientInfo"),
                InstructionFromTimingProgram::Freetext(text) => format!("Freetext: {}", text),
                InstructionFromTimingProgram::Advertisements => String::from("Advertisements"),
                InstructionFromTimingProgram::Clear => String::from("Clear"),
                InstructionFromTimingProgram::StartList => String::from("StartList"),
                InstructionFromTimingProgram::Timing => String::from("Timing"),
                InstructionFromTimingProgram::SetProperty => String::from("SetProperty"),
                InstructionFromTimingProgram::Results => String::from("Results"),
                InstructionFromTimingProgram::ResultsUpdate => String::from("ResultsUpdate"),
                InstructionFromTimingProgram::ServerInfo => String::from("ServerInfo"),
                InstructionFromTimingProgram::SendFrame(_) => String::from("SendFrame"),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InstructionToTimingProgram {
    SendServerInfo,
    SendFrame(Vec<u8>), // stores the frame data
}

#[derive(Clone)]
pub struct InstructionCommunicationChannel {
    args: Args,
    inbound_sender: Sender<IncomingInstruction>,
    inbound_receiver: Receiver<IncomingInstruction>,
    outbound_sender: Sender<InstructionToTimingProgram>,
    outbound_receiver: Receiver<InstructionToTimingProgram>,
}
impl InstructionCommunicationChannel {
    pub fn new(args: &Args) -> Self {
        let (is, ir) = async_channel::bounded::<IncomingInstruction>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        let (os, or) = async_channel::bounded::<InstructionToTimingProgram>(
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

    pub fn take_in_command_from_timing_program(
        &self,
        inst: InstructionFromTimingProgram,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromTimingProgram(inst))
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

    pub fn send_out_command(&self, inst: InstructionToTimingProgram) -> Result<(), String> {
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
    ) -> Result<Result<InstructionToTimingProgram, RecvError>, Elapsed> {
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
