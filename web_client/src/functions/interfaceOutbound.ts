import { DayTime } from "./interfaceInbound";
import { DatabaseStaticState, TimingSettings } from "./interfaceShared";

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
