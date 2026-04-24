use crate::{
    client::{frametime::FrametimeReport, TimingSettings},
    database::{DatabaseStaticState, PermanentlyStoredDataset},
    productkey::ProductKey,
    server::{
        bib_detection::{BibDataPoint, BibEntryModeData, BibEquivalence, DisplayEntry},
        camera_program_types::{
            Athlete, AthleteWithMetadata, CompetitorEvaluated, HeatAssignment, HeatData,
            HeatFinish, HeatIntermediate, HeatMeta, HeatResult, HeatStart, HeatStartList, HeatWind,
        },
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
    RequestDevMode,
    RequestPassword,
    RequestLicense,
    RequestConnectionStates,
    SelectHeatForBibMode(Uuid),
    RequestBibEntryModeData,
    SendHeatDataToDisplay(Uuid),
    AddBibEquivalence(BibEquivalence),
    DeleteBibEquivalence(BibEquivalence),
    RecordBibRound(u32),
    // DEV calls
    DevReset,
    DevStartRace(HeatStart),
    DevSendStartList(HeatStartList),
    DevSendIntermediateSignal(HeatIntermediate),
    DevSendFinishSignal(HeatFinish),
    DevSendEvaluated(CompetitorEvaluated),
    DevSendResultList(HeatResult),
    DevSendWind(HeatWind),
    DevRequestMainHeatStartList,
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
pub struct ConnectionState {
    pub try_connect_to_wind: bool, // wind connection status is calculated depending on ingoing wind packets that get pushed
    pub wind_address_with_port: String,
    pub wind_connected: bool,
    pub try_conect_to_display_client: bool, // display client connection is exchanged on push because is carries additional information
    pub display_client_address_with_port: String,
    pub bib_connected: bool,
    pub try_connect_to_bib: bool,
    pub bib_address_with_port: String,
    pub idcapture_connected: bool,
    pub try_to_connect_to_idcapture: bool,
    pub idcapture_address_with_port: String,
    pub camera_program_connected: bool,
    pub camera_program_connected_on_timing_port: bool,
    pub camera_program_connected_on_data_port: bool,
    pub camera_program_connected_on_xml_port: bool,
    pub camera_program_timing_port: String,
    pub camera_program_data_port: String,
    pub camera_program_xml_port: String,
    pub try_to_connect_to_camera_program: bool,
    pub camera_program_address: String,
    pub display_passthrough_connected: bool,
    pub try_to_connect_to_display_passthrough: bool,
    pub display_passthrough_address: String,
    pub timing_program_is_connected: bool,
    pub listening_to_timing_program: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, TypescriptSerializable)]
#[serde(tag = "type", content = "data")]
pub enum MessageToWebControl {
    DatabaseStaticState(DatabaseStaticState),
    DisplayClientState(DisplayClientState),
    HeatsMeta(Vec<HeatMeta>),
    Logs(Vec<PermanentlyStoredDataset>),
    HeatDataMessage(HeatData),
    HeatDataSelectionForBibMode(Option<BibEntryModeData>),
    TimingSettingsState(TimingSettings),
    WindMeasurements(Vec<WindMeasurement>),
    CurrentDisplayFrame(Vec<u8>), // gets handled extra and sent as binary data
    AthletesData(Vec<AthleteWithMetadata>),
    PDFConfigurationSettingsData(Vec<PDFConfigurationSetting>),
    MainHeat(HeatData),
    VersionMismatch((String, String)),
    FrametimeReport(FrametimeReport),
    DevModeStatus(bool),
    Password(String),
    Licensed(Option<ProductKey>),
    StaticConfigurationNotInitialized,
    ConnectionState(ConnectionState),
    BibRoundRecorded(BibDataPoint),
    // DEV test calls
    DevMainHeatStartList(HeatStartList),
}
