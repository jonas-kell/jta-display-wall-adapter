// server Datatype formats (keep in sync with "./../../../src/server/camera_program_datatypes.rs")

import { Athlete, DatabaseStaticState, HeatAssignment, PDFConfigurationSetting, TimingSettings, Uuid } from "./interfaceShared";

export type HeatMeta = {
    id: string;
    name: string;
    number: number;
    scheduled_start_time_string: string;
};

export type NaiveDateTime = string;

export type RaceWind = {
    /** - is headwind, + is backwind */
    back_wind: boolean;
    whole_number_part: number;
    fraction_part: number; // 0–9
};

export type DayTime = {
    hours: number;
    minutes: number;
    seconds: number;
    fractional_part_in_ten_thousands: number | null;
};

export type RaceTime = {
    hours: number | null;
    minutes: number | null;
    seconds: number;
    tenths: number | null;
    hundrets: number | null;
    thousands: number | null;
    ten_thousands: number | null;
};

export type DisqualificationReason =
    | { type: "Disqualified" }
    | { type: "DidNotStart" }
    | { type: "DidNotFinish" }
    | { type: "Canceled" }
    | { type: "Other"; value: string };

export type DifferenceToCandidate = { type: "Winner" } | { type: "Difference"; value: RaceTime };

export type HeatCompetitor = {
    id: string;
    lane: number;
    bib: number;
    class: string;
    last_name: string;
    first_name: string;
    nation: string;
    club: string;
    gender: string;
    disqualified: DisqualificationReason | null;
};

export type HeatCompetitorResult = {
    competitor: HeatCompetitor;
    distance: number;
    rank: number;
    runtime: RaceTime;
    runtime_full_precision: RaceTime;
    difference_to_winner: DifferenceToCandidate;
    difference_to_previous: DifferenceToCandidate;
    finish_time: DayTime;
};

export type CompetitorEvaluated = {
    application: string;
    version: string;
    generated: NaiveDateTime;
    id: Uuid;
    competitor_result: HeatCompetitorResult;
};

export type HeatStart = {
    application: string;
    version: string;
    generated: NaiveDateTime;
    id: Uuid;
    time: DayTime;
};

export type HeatFinish = {
    application: string;
    version: string;
    generated: NaiveDateTime;
    id: Uuid;
    time: DayTime;
    race_time: RaceTime;
};

export type HeatIntermediate = {
    application: string;
    version: string;
    generated: NaiveDateTime;
    id: Uuid;
    time: DayTime;
    intermediate_time_at: RaceTime;
};

export type HeatWind = {
    application: string;
    version: string;
    generated: NaiveDateTime;
    id: Uuid;
    wind: RaceWind;
};

export type HeatStartList = {
    name: string;
    id: Uuid;
    nr: number;
    session_nr: number;
    distance_meters: number;
    scheduled_start_time: DayTime;
    competitors: HeatCompetitor[];
};

export type HeatResult = {
    id: Uuid;
    name: string;
    distance_meters: number;
    start_time: DayTime;
    wind: RaceWind | null;
    competitors_evaluated: HeatCompetitorResult[];
    competitors_left_to_evaluate: HeatCompetitor[];
};

export type HeatData = {
    meta: HeatMeta;
    start_list: HeatStartList;
    start: HeatStart | null;
    intermediates: HeatIntermediate[] | null;
    wind: HeatWind | null;
    finish: HeatFinish | null;
    evaluations: CompetitorEvaluated[] | null;
    result: HeatResult | null;
};

export enum WindMeasurementType {
    Polling = "Polling",
    UnidentifiedMeasurement = "UnidentifiedMeasurement",
    Race10s = "Race10s",
    Race13s = "Race13s",
    Jump5s = "Jump5s",
    Other8s = "Other8s",
    Other12s = "Other12s",
}

export type WindMeasurement = {
    wind: RaceWind;
    probable_measurement_type: WindMeasurementType;
    time: DayTime | null;
};

export type AthleteWithMetadata = {
    athlete: Athlete;
    heat_assignments: HeatAssignment[];
    heats_from_assignments: [HeatCompetitorResult | null, HeatAssignment, HeatData][];
};

// message formats

export type DisplayClientStateState = {
    alive: boolean;
    external_passthrough_mode: boolean;
    can_switch_mode: boolean;
};

export type DatabaseStaticStateMessage = {
    type: "DatabaseStaticState";
    data: DatabaseStaticState;
};

export type DisplayClientState = {
    type: "DisplayClientState";
    data: DisplayClientStateState;
};

export type HeatsMeta = {
    type: "HeatsMeta";
    data: HeatMeta[];
};

export type LogEntry = {
    name_key: string;
    stored_at: string;
    data: string;
};

export type Logs = {
    type: "Logs";
    data: LogEntry[];
};

export type HeatDataMessage = {
    type: "HeatDataMessage";
    data: HeatData;
};

export type TimingSettingsState = {
    type: "TimingSettingsState";
    data: TimingSettings;
};

export type WindMeasurements = {
    type: "WindMeasurements";
    data: WindMeasurement[];
};

export type CurrentDisplayFrame = {
    type: "CurrentDisplayFrame";
    data: number[];
};

export type AthletesData = {
    type: "AthletesData";
    data: AthleteWithMetadata[];
};

export type PDFConfigurationSettingsData = {
    type: "PDFConfigurationSettingsData";
    data: PDFConfigurationSetting[];
};

export type MainHeat = {
    type: "MainHeat";
    data: HeatData;
};

export type Unknown = {
    type: "Unknown";
    data: unknown;
};

export enum InboundMessageType {
    DisplayClientState = "DisplayClientState",
    Unknown = "Unknown",
    HeatsMeta = "HeatsMeta",
    Logs = "Logs",
    HeatDataMessage = "HeatDataMessage",
    TimingSettingsState = "TimingSettingsState",
    WindMeasurements = "WindMeasurements",
    CurrentDisplayFrame = "CurrentDisplayFrame",
    DatabaseStaticState = "DatabaseStaticState",
    AthletesData = "AthletesData",
    PDFConfigurationSettingsData = "PDFConfigurationSettingsData",
    MainHeat = "MainHeat",
}

export type InboundMessage =
    | DatabaseStaticStateMessage
    | DisplayClientState
    | HeatsMeta
    | Logs
    | HeatDataMessage
    | TimingSettingsState
    | WindMeasurements
    | CurrentDisplayFrame
    | AthletesData
    | PDFConfigurationSettingsData
    | MainHeat
    | Unknown;

export function parseMessage(json: unknown): InboundMessage {
    if (typeof json !== "object" || json === null) {
        throw new Error("Invalid message: not an object");
    }

    const obj = json as Record<string, unknown>;

    switch (obj.type) {
        case InboundMessageType.DisplayClientState:
            return { type: "DisplayClientState", data: obj.data } as DisplayClientState;
        case InboundMessageType.HeatsMeta:
            return { type: "HeatsMeta", data: obj.data } as HeatsMeta;
        case InboundMessageType.Logs:
            return { type: "Logs", data: obj.data } as Logs;
        case InboundMessageType.HeatDataMessage:
            return { type: "HeatDataMessage", data: obj.data } as HeatDataMessage;
        case InboundMessageType.TimingSettingsState:
            return { type: "TimingSettingsState", data: obj.data } as TimingSettingsState;
        case InboundMessageType.WindMeasurements:
            return { type: "WindMeasurements", data: obj.data } as WindMeasurements;
        case InboundMessageType.CurrentDisplayFrame:
            return { type: "CurrentDisplayFrame", data: obj.data } as CurrentDisplayFrame;
        case InboundMessageType.DatabaseStaticState:
            return { type: "DatabaseStaticState", data: obj.data } as DatabaseStaticStateMessage;
        case InboundMessageType.AthletesData:
            return { type: "AthletesData", data: obj.data } as AthletesData;
        case InboundMessageType.PDFConfigurationSettingsData:
            return { type: "PDFConfigurationSettingsData", data: obj.data } as PDFConfigurationSettingsData;
        case InboundMessageType.MainHeat:
            return { type: "MainHeat", data: obj.data } as MainHeat;

        default:
            return { type: "Unknown", data: json } as Unknown;
    }
}
