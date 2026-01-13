use crate::{
    client::TimingSettings,
    database::{DatabaseStaticState, PermanentlyStoredDataset},
    server::{
        bib_detection::DisplayEntry,
        camera_program_types::{Athlete, AthleteWithMetadata, HeatAssignment, HeatData, HeatMeta},
    },
    times::DayTime,
    wind::format::WindMeasurement,
};
use chrono::NaiveDateTime;
use rust_to_ts_types::TypescriptSerializable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, TypescriptSerializable)]
#[serde(tag = "type", content = "data")]
pub enum MessageFromWebControl {
    Idle,
    Advertisements,
    FreeText(String),
    RequestDisplayClientState,
    SwitchMode,
    GetHeats,
    GetMainHeat,
    GetLogs(u32),
    SelectHeat(String),
    Timing,
    StartList,
    ResultList,
    UpdateTimingSettings(TimingSettings),
    RequestTimingSettings,
    Clock(DayTime),
    RequestWindValues(WindValueRequestDateContainer),
    InitStaticDatabaseState(DatabaseStaticState),
    RequestStaticDatabaseState,
    ExportDataToFile,
    CreateAthlete(Athlete),
    DeleteAthlete(Uuid),
    CreateHeatAssignment(HeatAssignment),
    DeleteHeatAssignment(i32),
    RequestAthletes,
    StorePDFConfigurationSetting(PDFConfigurationSetting),
    DeletePDFConfigurationSetting(Uuid),
    RequestPDFConfigurationSettings,
    DeleteCompetitorEvaluated(DayTime), // to target the correct HeatCompetitorResult, as here are no ids
    SendDebugDisplayCommand(DisplayEntry),
}

#[derive(Debug, Serialize, Deserialize, Clone, TypescriptSerializable)]
pub enum PDFSettingFor {
    Bib,
    Certificate,
}

#[derive(Debug, Serialize, Deserialize, Clone, TypescriptSerializable)]
pub struct PDFConfigurationSetting {
    pub id: Uuid,
    pub setting_for: PDFSettingFor,
    pub pos_x: f64,
    pub pos_y: f64,
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub centered: bool,
    pub content: PDFConfigurationContent,
}

#[derive(Debug, Serialize, Deserialize, Clone, TypescriptSerializable)]
#[serde(tag = "type", content = "data")]
pub enum PDFConfigurationContent {
    PDFConfigurationContentText {
        text: String,
    },
    PDFConfigurationContentReference {
        reference: String,
        reference_content: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, TypescriptSerializable)]
pub struct WindValueRequestDateContainer {
    pub from: NaiveDateTime,
    pub to: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, TypescriptSerializable)]
pub struct DisplayClientState {
    pub alive: bool,
    pub external_passthrough_mode: bool,
    pub can_switch_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, TypescriptSerializable)]
#[serde(tag = "type", content = "data")]
pub enum MessageToWebControl {
    DatabaseStaticState(DatabaseStaticState),
    DisplayClientState(DisplayClientState),
    HeatsMeta(Vec<HeatMeta>),
    Logs(Vec<PermanentlyStoredDataset>),
    HeatDataMessage(HeatData),
    TimingSettingsState(TimingSettings),
    WindMeasurements(Vec<WindMeasurement>),
    CurrentDisplayFrame(Vec<u8>), // gets handled extra and sent as binary data
    AthletesData(Vec<AthleteWithMetadata>),
    PDFConfigurationSettingsData(Vec<PDFConfigurationSetting>),
    MainHeat(HeatData),
    VersionMismatch((String, String)),
}
