use crate::{
    idcapture::format::IDCaptureMessage,
    interface::MessageFromClientToServer,
    server::{
        bib_detection::MessageFromBibServer,
        camera_program_types::{
            CompetitorEvaluated, HeatFalseStart, HeatFinish, HeatIntermediate, HeatResult,
            HeatStart, HeatStartList, HeatWind, HeatWindMissing,
        },
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
    FromExternalDisplayProgram(InstructionFromExternalDisplayProgram),
    FromCameraProgram(InstructionFromCameraProgram),
    FromWebControl(MessageFromWebControl),
    FromWindServer(WindMessageBroadcast),
    FromBibServer(MessageFromBibServer),
    FromIdcaptureServer(IDCaptureMessage),
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
                IncomingInstruction::FromExternalDisplayProgram(dsi) =>
                    format!("FromExternalDisplayProgram: {}", dsi),
                IncomingInstruction::FromCameraProgram(cpi) =>
                    format!("FromCameraProgram: {:?}", cpi),
                IncomingInstruction::FromWebControl(wci) => format!("FromWebControl: {:?}", wci),
                IncomingInstruction::FromWindServer(wmb) => format!("FromWindServer: {:?}", wmb),
                IncomingInstruction::FromBibServer(bm) => format!("FromBibServer: {:?}", bm),
                IncomingInstruction::FromIdcaptureServer(idcm) =>
                    format!("FromIdcaptureServer: {:?}", idcm),
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
            }
        )
    }
}

#[derive(Serialize, Deserialize)]
pub enum InstructionFromExternalDisplayProgram {
    ServerInfo,
    Frame(Vec<u8>),
}
impl Display for InstructionFromExternalDisplayProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InstructionFromExternalDisplayProgram::ServerInfo => String::from("ServerInfo"),
                InstructionFromExternalDisplayProgram::Frame(_) => String::from("Frame"),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InstructionToTimingProgram {
    SendServerInfo,
    SendFrame(Vec<u8>), // stores the frame data
}
