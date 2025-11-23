// server Datatype formats (keep in sync with "./../../../src/server/camera_program_datatypes.rs")

export type HeatMeta = {
    id: string;
    name: string;
    number: number;
    scheduled_start_time_string: string;
};

// message formats

export type DisplayClientStateState = {
    alive: boolean;
    external_passthrough_mode: boolean;
    can_switch_mode: boolean;
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

export type Unknown = {
    type: "Unknown";
    data: unknown;
};

export enum InboundMessageType {
    DisplayClientState = "DisplayClientState",
    Unknown = "Unknown",
    HeatsMeta = "HeatsMeta",
    Logs = "Logs",
}

export type InboundMessage = DisplayClientState | HeatsMeta | Logs | Unknown;

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

        default:
            return { type: "Unknown", data: json } as Unknown;
    }
}
