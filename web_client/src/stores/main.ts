import { defineStore } from "pinia";
import { computed, nextTick, ref, watch } from "vue";
import { WS_URL } from "../functions/environment";
import {
    Advertisements,
    Clock,
    FreeText,
    GetHeats,
    GetLogs,
    Idle,
    RequestDisplayClientState,
    RequestTimingSettings,
    RequestWindValues,
    ResultList,
    SelectHeat,
    StartList,
    SwitchMode,
    Timing,
    UpdateTimingSettings,
    WindValueRequestDateContainer,
} from "../functions/interfaceOutbound";
import {
    DayTime,
    HeatData,
    HeatMeta,
    InboundMessageType,
    LogEntry,
    parseMessage,
    RaceWind,
    WindMeasurement,
} from "../functions/interfaceInbound";
import { CircularBuffer } from "../functions/circularBuffer";
import { TimingSettings } from "../functions/interfaceShared";
import { dayTimeStringRepr, windStringRepr } from "../functions/representation";

function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

const WIND_MESSAGE = "Not synced (start any Race in Camera Program to fix)";

export default defineStore("main", () => {
    const connected = ref(false);
    const displayConnected = ref(false);
    const displayExternalPassthrough = ref(false);
    const displayCanSwitchModeInternal = ref(false);

    let reconnecting = false;
    let ws = null as null | WebSocket;

    watch(connected, () => {
        if (!connected.value) {
            displayConnected.value = false;
        }
    });

    const WIND_CONNECTION_TIMEOUT = 5_000; // ms
    const lastWindPing = ref<number>(Date.now() - 2 * WIND_CONNECTION_TIMEOUT);
    const windTime = ref<string>("");
    const windValue = ref<string>("");
    const now = ref<number>(Date.now());
    window.setInterval(() => {
        now.value = Date.now();
    }, 500);
    function refreshWindConnectionTimer() {
        lastWindPing.value = Date.now();
    }
    const windServerConnected = computed<boolean>(() => {
        return now.value - lastWindPing.value <= WIND_CONNECTION_TIMEOUT;
    });
    watch(windServerConnected, () => {
        if (!windServerConnected.value) {
            windTime.value = WIND_MESSAGE;
            windValue.value = "----";
        }
    });

    const logEntriesRolling = new CircularBuffer<LogEntry>(10);
    const requestedWindMeasurements = ref([] as WindMeasurement[]);

    function handleWSMessage(ev: any) {
        let msg = parseMessage(JSON.parse(ev.data));

        switch (msg.type) {
            case InboundMessageType.DisplayClientState:
                displayConnected.value = msg.data.alive;
                displayExternalPassthrough.value = msg.data.external_passthrough_mode;
                displayCanSwitchModeInternal.value = msg.data.can_switch_mode;
                return;
            case InboundMessageType.HeatsMeta:
                heatsMetaResult.value = msg.data;
                heatsMetaResult.value.sort((a, b) => {
                    return a.scheduled_start_time_string.localeCompare(b.scheduled_start_time_string);
                });
                return;
            case InboundMessageType.Logs:
                const entArr = msg.data;
                if (entArr.length == 1) {
                    const new_entry = entArr[0];
                    // constant wind spams logs
                    if (detectWindPolling(new_entry.data)) {
                        refreshWindConnectionTimer();
                    } else {
                        // entry per push, not requested
                        logEntriesRolling.unshift(new_entry);
                        logEntries.value = logEntriesRolling.toArray();
                    }
                } else {
                    logEntries.value = msg.data;
                }
                sendGetHeatsCommand();
                sendSelectHeatCommandInternal();
                return;
            case InboundMessageType.HeatDataMessage:
                selectedHeat.value = msg.data;
                return;
            case InboundMessageType.TimingSettingsState:
                timingSettingsBeingChanged.value = true;
                nextTick(() => {
                    timingSettings.value = msg.data;
                    nextTick(() => {
                        timingSettingsBeingChanged.value = false;
                    });
                });
                return;
            case InboundMessageType.WindMeasurements:
                requestedWindMeasurements.value = msg.data;
                return;
            case InboundMessageType.Unknown:
                console.error("Received unknown message type:", msg.data);
                return;
        }
        console.error("Received unhandled message type:", msg);
    }

    async function initWS() {
        if (reconnecting) {
            // prevent multiple runs
            console.log("Already reconnecting");
            return;
        }
        reconnecting = true;
        console.log("(Re)connecting WS");

        if (ws) {
            try {
                ws.close();
            } catch (_) {}
            ws = null;
            await sleep(500);
        }

        ws = new WebSocket(WS_URL);
        ws.onerror = async () => {
            connected.value = false;
            reconnecting = false;
            await sleep(2000);
            console.log("Retry connecting to socket after error");
            initWS();
        };
        ws.onopen = () => {
            connected.value = true;
            reconnecting = false;

            console.log("Socket connected");

            // this is kind of an init also, as this gets requested on connection establish:
            sendGetHeatsCommand();
            sendRequestTimingSettingsCommand();

            // only assign the handlers if actually open
            if (ws) {
                ws.onclose = async () => {
                    connected.value = false;

                    console.log("Socket closed.");
                    await sleep(1000);
                    console.log("Socket closed. Reconnecting");
                    initWS();
                };
                ws.onmessage = handleWSMessage;
            }

            // immediately request client state
            const packet: RequestDisplayClientState = {
                type: "RequestDisplayClientState",
            };
            sendWSCommand(JSON.stringify(packet));
        };
    }
    initWS();

    function sendWSCommand(dat: string) {
        if (ws && connected.value) {
            ws.send(dat);
        }
        if (!ws) {
            console.error("Websocket was never connected");
        }
        if (!connected.value) {
            console.error("Websocket is not connected");
        }
    }

    function sendAdvertisementsCommand() {
        const packet: Advertisements = {
            type: "Advertisements",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendTimingCommand() {
        const packet: Timing = {
            type: "Timing",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendStartListCommand() {
        const packet: StartList = {
            type: "StartList",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendResultListCommand() {
        const packet: ResultList = {
            type: "ResultList",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendIdleCommand() {
        const packet: Idle = {
            type: "Idle",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendFreetextCommand(payload: string) {
        const packet: FreeText = {
            type: "FreeText",
            data: payload,
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendSwitchModeCommand() {
        displayCanSwitchModeInternal.value = false; // will be reset on updating message
        const packet: SwitchMode = {
            type: "SwitchMode",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendGetWindValuesCommand(data: WindValueRequestDateContainer) {
        const packet: RequestWindValues = {
            type: "RequestWindValues",
            data,
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendClockCommand() {
        const now = new Date();
        const hours = now.getHours();
        const minutes = now.getMinutes();
        const seconds = now.getSeconds();

        const packet: Clock = {
            type: "Clock",
            data: {
                fractional_part_in_ten_thousands: now.getMilliseconds() * 10,
                hours,
                minutes,
                seconds,
            },
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendGetHeatsCommand() {
        const packet: GetHeats = {
            type: "GetHeats",
        };
        sendWSCommand(JSON.stringify(packet));
    }
    const heatsMetaResult = ref([] as HeatMeta[]);
    function sendRequestTimingSettingsCommand() {
        const packet: RequestTimingSettings = {
            type: "RequestTimingSettings",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendGetLogsCommand(how_many: number) {
        if (how_many < 0) {
            how_many = 1;
        }
        how_many = Math.floor(how_many);

        const packet: GetLogs = {
            type: "GetLogs",
            data: how_many,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    const logEntries = ref([] as LogEntry[]);

    function sendSelectHeatCommand(id: string) {
        selectHeatId = id;
        sendSelectHeatCommandInternal();
    }
    function sendSelectHeatCommandInternal() {
        if (selectHeatId) {
            const packet: SelectHeat = {
                type: "SelectHeat",
                data: selectHeatId,
            };
            sendWSCommand(JSON.stringify(packet));
        }
    }
    let selectHeatId = null as null | string;
    const selectedHeat = ref(null as null | HeatData);

    const timingSettings = ref(null as null | TimingSettings);
    const timingSettingsBeingChanged = ref(true); // initial change
    watch(
        timingSettings,
        () => {
            if (timingSettings.value && timingSettingsBeingChanged.value == false) {
                timingSettingsBeingChanged.value = true;
                const packet: UpdateTimingSettings = {
                    type: "UpdateTimingSettings",
                    data: timingSettings.value,
                };
                sendWSCommand(JSON.stringify(packet));
            }
        },
        {
            deep: true,
        }
    );

    function detectWindPolling(data: string): boolean {
        const parsed = JSON.parse(data);

        if (Object.keys(parsed).includes("time")) {
            const timeElem: null | DayTime = parsed["time"];

            if (timeElem == null) {
                windTime.value = WIND_MESSAGE;
            } else {
                windTime.value = dayTimeStringRepr(timeElem);
            }
        }

        if (Object.keys(parsed).includes("wind")) {
            const windVal: RaceWind = parsed["wind"];

            windValue.value = windStringRepr(windVal);
        }

        if (Object.keys(parsed).includes("probable_measurement_type")) {
            if (parsed["probable_measurement_type"] == "Polling") {
                return true;
            }
        }

        return false;
    }

    const displayCanSwitchMode = computed(() => {
        return displayCanSwitchModeInternal.value && displayConnected.value;
    });
    const canEditTimingSettings = computed(() => {
        return timingSettingsBeingChanged.value && timingSettings.value != null;
    });

    return {
        connected,
        sendSwitchModeCommand,
        sendAdvertisementsCommand,
        sendIdleCommand,
        sendFreetextCommand,
        sendGetHeatsCommand,
        sendGetLogsCommand,
        sendSelectHeatCommand,
        sendTimingCommand,
        sendStartListCommand,
        sendResultListCommand,
        sendClockCommand,
        sendGetWindValuesCommand,
        canEditTimingSettings,
        timingSettings,
        selectedHeat,
        logEntries,
        heatsMetaResult,
        displayConnected,
        displayExternalPassthrough,
        displayCanSwitchMode,
        windServerConnected,
        windTime,
        windValue,
        requestedWindMeasurements,
    };
});
