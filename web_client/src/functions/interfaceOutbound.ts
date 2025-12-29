import { DayTime } from "./interfaceInbound";
import { Athlete, DatabaseStaticState, HeatAssignment, PDFConfigurationSetting, TimingSettings, Uuid } from "./interfaceShared";

export type Idle = {
    type: "Idle";
};

export type Advertisements = {
    type: "Advertisements";
};

export type FreeText = {
    type: "FreeText";
    data: string;
};

export type RequestDisplayClientState = {
    type: "RequestDisplayClientState";
};

export type SwitchMode = {
    type: "SwitchMode";
};

export type GetHeats = {
    type: "GetHeats";
};

export type GetMainHeat = {
    type: "GetMainHeat";
};

export type GetLogs = {
    type: "GetLogs";
    data: number;
};

export type SelectHeat = {
    type: "SelectHeat";
    data: string;
};

export type Timing = {
    type: "Timing";
};

export type StartList = {
    type: "StartList";
};

export type ResultList = {
    type: "ResultList";
};

export type Clock = {
    type: "Clock";
    data: DayTime;
};

export type UpdateTimingSettings = {
    type: "UpdateTimingSettings";
    data: TimingSettings;
};

export type RequestTimingSettings = {
    type: "RequestTimingSettings";
};

export type WindValueRequestDateContainer = {
    from: string;
    to: string;
};

export type RequestWindValues = {
    type: "RequestWindValues";
    data: WindValueRequestDateContainer;
};

export type InitStaticDatabaseState = {
    type: "InitStaticDatabaseState";
    data: DatabaseStaticState;
};

export type RequestStaticDatabaseState = {
    type: "RequestStaticDatabaseState";
};

export type ExportDataToFile = {
    type: "ExportDataToFile";
};

export type CreateAthlete = {
    type: "CreateAthlete";
    data: Athlete;
};

export type DeleteAthlete = {
    type: "DeleteAthlete";
    data: Uuid;
};

export type CreateHeatAssignment = {
    type: "CreateHeatAssignment";
    data: HeatAssignment;
};

export type DeleteHeatAssignment = {
    type: "DeleteHeatAssignment";
    data: number;
};

export type RequestAthletes = {
    type: "RequestAthletes";
};

export type StorePDFConfigurationSetting = {
    type: "StorePDFConfigurationSetting";
    data: PDFConfigurationSetting;
};

export type DeletePDFConfigurationSetting = {
    type: "DeletePDFConfigurationSetting";
    data: Uuid;
};

export type RequestPDFConfigurationSettings = {
    type: "RequestPDFConfigurationSettings";
};

export type DeleteCompetitorEvaluated = {
    type: "DeleteCompetitorEvaluated";
    data: DayTime;
};
