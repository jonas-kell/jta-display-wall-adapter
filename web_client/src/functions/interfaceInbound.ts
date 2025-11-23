export type DisplayClientStateState = {
    alive: boolean;
    external_passthrough_mode: boolean;
    can_switch_mode: boolean;
};

export type DisplayClientState = {
    type: "DisplayClientState";
    data: DisplayClientStateState;
};

export type HeatStarts = {
    type: "HeatStarts";
    data: any;
};

export type Unknown = {
    type: "Unknown";
    data: unknown;
};

export enum InboundMessageType {
    DisplayClientState = "DisplayClientState",
    Unknown = "Unknown",
    HeatStarts = "HeatStarts",
}

export type InboundMessage = DisplayClientState | HeatStarts | Unknown;

export function parseMessage(json: unknown): InboundMessage {
    if (typeof json !== "object" || json === null) {
        throw new Error("Invalid message: not an object");
    }

    const obj = json as Record<string, unknown>;

    switch (obj.type) {
        case InboundMessageType.DisplayClientState:
            return { type: "DisplayClientState", data: obj.data } as DisplayClientState;
        case InboundMessageType.HeatStarts:
            return { type: "HeatStarts", data: obj.data } as HeatStarts;

        default:
            return { type: "Unknown", data: json } as Unknown;
    }
}
