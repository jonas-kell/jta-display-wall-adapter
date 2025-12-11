use crate::{
    interface::MessageFromClientToServer,
    server::camera_program_types::{
        CompetitorEvaluated, HeatFalseStart, HeatFinish, HeatIntermediate, HeatResult, HeatStart,
        HeatStartList, HeatWind, HeatWindMissing,
    },
    times::{DayTime, RaceTime},
    webserver::MessageFromWebControl,
    wind::format::WindMessageBroadcast,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub enum IncomingInstruction {
    FromClient(MessageFromClientToServer),
    FromTimingProgram(InstructionFromTimingProgram),
    FromCameraProgram(InstructionFromCameraProgram),
    FromWebControl(MessageFromWebControl),
    FromWindServer(WindMessageBroadcast),
}
impl Display for IncomingInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                IncomingInstruction::FromClient(mfcts) => format!("FromClient: {:?}", mfcts),
                IncomingInstruction::FromTimingProgram(tci) =>
                    format!("FromTimingProgram: {}", tci),
                IncomingInstruction::FromCameraProgram(cpi) =>
                    format!("FromCameraProgram: {:?}", cpi),
                IncomingInstruction::FromWebControl(wci) => format!("FromWebControl: {:?}", wci),
                IncomingInstruction::FromWindServer(wmb) => format!("FromWindServer: {:?}", wmb),
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
    HeatWindMissing(HeatWindMissing),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InstructionToTimingProgram {
    SendServerInfo,
    SendFrame(Vec<u8>), // stores the frame data
}
