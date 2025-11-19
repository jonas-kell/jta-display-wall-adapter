export type DisplayClientStateState = {
    alive: boolean;
    external_passthrough_mode: boolean;
};

export type DisplayClientState = {
    type: "DisplayClientState";
    data: DisplayClientStateState;
};

export type Unknown = {
    type: "Unknown";
    data: unknown;
};

export enum InboundMessageType {
    DisplayClientState = "DisplayClientState",
    Unknown = "Unknown",
}

export type InboundMessage = DisplayClientState | Unknown;

export function parseMessage(json: unknown): InboundMessage {
    if (typeof json !== "object" || json === null) {
        throw new Error("Invalid message: not an object");
    }

    const obj = json as Record<string, unknown>;

    switch (obj.type) {
        case InboundMessageType.DisplayClientState:
            return { type: "DisplayClientState", data: obj.data } as DisplayClientState;

        default:
            return { type: "Unknown", data: json } as Unknown;
    }
}
