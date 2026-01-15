import { defineStore } from "pinia";
import { computed, nextTick, ref, watch } from "vue";
import { WS_URL } from "../functions/environment";
import {
    WindValueRequestDateContainer,
    AthleteWithMetadata,
    DayTime,
    HeatData,
    HeatMeta,
    RaceWind,
    WindMeasurement,
    Athlete,
    DatabaseStaticState,
    DisplayEntry,
    HeatAssignment,
    PDFConfigurationSetting,
    TimingSettings,
    Uuid,
    MessageToWebControl,
    PermanentlyStoredDataset,
    MessageFromWebControlRequestDisplayClientState,
    MessageFromWebControlAdvertisements,
    MessageFromWebControlTiming,
    MessageFromWebControlStartList,
    MessageFromWebControlExportDataToFile,
    MessageFromWebControlResultList,
    MessageFromWebControlIdle,
    MessageFromWebControlFreeText,
    MessageFromWebControlSwitchMode,
    MessageFromWebControlRequestWindValues,
    MessageFromWebControlClock,
    MessageFromWebControlRequestStaticDatabaseState,
    MessageFromWebControlInitStaticDatabaseState,
    MessageFromWebControlRequestAthletes,
    MessageFromWebControlCreateAthlete,
    MessageFromWebControlDeleteAthlete,
    MessageFromWebControlDeleteCompetitorEvaluated,
    MessageFromWebControlStorePDFConfigurationSetting,
    MessageFromWebControlDeletePDFConfigurationSetting,
    MessageFromWebControlRequestPDFConfigurationSettings,
    MessageFromWebControlCreateHeatAssignment,
    MessageFromWebControlDeleteHeatAssignment,
    MessageFromWebControlSendDebugDisplayCommand,
    MessageFromWebControlGetHeats,
    MessageFromWebControlGetMainHeat,
    MessageFromWebControlRequestTimingSettings,
    MessageFromWebControlGetLogs,
    MessageFromWebControlSelectHeat,
    MessageFromWebControlUpdateTimingSettings,
    MessageFromWebControlRequestDevMode,
    MessageFromWebControl,
    HeatStartList,
    FrametimeReport,
} from "../generated/interface";
import { CircularBuffer } from "../functions/circularBuffer";
import { dayTimeStringRepr, imageURLfromBMPBytes, imageURLfromBMPBytesArray, windStringRepr } from "../functions/representation";

function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

const WIND_MESSAGE = "Not synced (start any Race in Camera Program to fix)";

export default defineStore("main", () => {
    const staticConfiguration = ref(null as null | DatabaseStaticState);
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

    const currentClientFrame = ref(null as null | string);

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

    const logEntriesRolling = new CircularBuffer<PermanentlyStoredDataset>(10);
    const requestedWindMeasurements = ref([] as WindMeasurement[]);
    const athletesData = ref([] as AthleteWithMetadata[]);
    const pdfConfigurationSettings = ref([] as PDFConfigurationSetting[]);
    const mainHeat = ref(null as null | HeatData);
    const devMode = ref(false);
    const devMainHeatStartList = ref(null as null | HeatStartList);
    const versionMismatchTriggered = ref(null as null | string);
    let frametimeReport = ref(null as null | FrametimeReport);

    function handleWSMessage(ev: MessageEvent) {
        if (ev.data instanceof Blob) {
            // Binary message (default browser behavior)
            ev.data.arrayBuffer().then((buf) => {
                currentClientFrame.value = imageURLfromBMPBytes(buf);
            });
            return;
        }

        if (ev.data instanceof ArrayBuffer) {
            currentClientFrame.value = imageURLfromBMPBytes(ev.data);
            return;
        }

        let msg = JSON.parse(ev.data) as MessageToWebControl;

        switch (msg.type) {
            case "DisplayClientState":
                displayConnected.value = msg.data.alive;
                displayExternalPassthrough.value = msg.data.external_passthrough_mode;
                displayCanSwitchModeInternal.value = msg.data.can_switch_mode;
                return;
            case "HeatsMeta":
                heatsMetaResult.value = msg.data;
                heatsMetaResult.value.sort((a, b) => {
                    return a.scheduled_start_time_string.localeCompare(b.scheduled_start_time_string);
                });
                return;
            case "Logs":
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
            case "HeatDataMessage":
                selectedHeat.value = msg.data;
                return;
            case "TimingSettingsState":
                timingSettingsBeingChanged.value = true;
                nextTick(() => {
                    timingSettings.value = msg.data;
                    nextTick(() => {
                        timingSettingsBeingChanged.value = false;
                    });
                });
                return;
            case "WindMeasurements":
                requestedWindMeasurements.value = msg.data;
                return;
            case "CurrentDisplayFrame":
                currentClientFrame.value = imageURLfromBMPBytesArray(msg.data);
                console.error("Binary data should not be sent over JSON channel, but binary channel");
                return;
            case "DatabaseStaticState":
                console.log("Initialized static database state");
                staticConfiguration.value = msg.data;
                return;
            case "AthletesData":
                athletesData.value = msg.data;
                return;
            case "PDFConfigurationSettingsData":
                pdfConfigurationSettings.value = msg.data;
                return;
            case "MainHeat":
                mainHeat.value = msg.data;
                return;
            case "VersionMismatch":
                const dbHasVersion = msg.data[0];
                const programHasVersion = msg.data[1];

                versionMismatchTriggered.value = `The database has the Version: ${dbHasVersion}, while the program currently has the version ${programHasVersion}. Not compatible!!`;
                return;
            case "DevModeStatus":
                devMode.value = msg.data;
                return;
            case "DevMainHeatStartList":
                devMainHeatStartList.value = msg.data;
                return;
            case "FrametimeReport":
                frametimeReport.value = msg.data;
                return;
            default:
                console.error("Received unknown message type:", msg);
                const _exhaustive: never = msg;
                return _exhaustive;
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
            sendRequestStaticConfigCommand();
            sendGetMainHeatCommand();
            sendGetHeatsCommand();
            sendRequestTimingSettingsCommand();
            sendRequestAthletesCommand();
            sendRequestAllPDFSettingsCommand();
            sendRequestAllPDFSettingsCommand();
            sendRequestDevModeStatusCommand();

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
            const packet: MessageFromWebControlRequestDisplayClientState = {
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
        const packet: MessageFromWebControlAdvertisements = {
            type: "Advertisements",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendTimingCommand() {
        const packet: MessageFromWebControlTiming = {
            type: "Timing",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendStartListCommand() {
        const packet: MessageFromWebControlStartList = {
            type: "StartList",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendExportToFileCommand() {
        const packet: MessageFromWebControlExportDataToFile = {
            type: "ExportDataToFile",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendResultListCommand() {
        const packet: MessageFromWebControlResultList = {
            type: "ResultList",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendIdleCommand() {
        const packet: MessageFromWebControlIdle = {
            type: "Idle",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendFreetextCommand(payload: string) {
        const packet: MessageFromWebControlFreeText = {
            type: "FreeText",
            data: payload,
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendSwitchModeCommand() {
        displayCanSwitchModeInternal.value = false; // will be reset on updating message
        const packet: MessageFromWebControlSwitchMode = {
            type: "SwitchMode",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendGetWindValuesCommand(data: WindValueRequestDateContainer) {
        const packet: MessageFromWebControlRequestWindValues = {
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

        const packet: MessageFromWebControlClock = {
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

    function sendRequestStaticConfigCommand() {
        const packet: MessageFromWebControlRequestStaticDatabaseState = {
            type: "RequestStaticDatabaseState",
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendStaticallyConfigureServerCommand(data: DatabaseStaticState) {
        const packet: MessageFromWebControlInitStaticDatabaseState = {
            type: "InitStaticDatabaseState",
            data: data,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendRequestAthletesCommand() {
        const packet: MessageFromWebControlRequestAthletes = {
            type: "RequestAthletes",
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendUpsertAthleteCommand(athlete: Athlete) {
        const packet: MessageFromWebControlCreateAthlete = {
            type: "CreateAthlete",
            data: athlete,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendDeleteAthleteCommand(id: Uuid) {
        const packet: MessageFromWebControlDeleteAthlete = {
            type: "DeleteAthlete",
            data: id,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendDeleteCompetitorEvaluatedCommand(dt: DayTime) {
        const packet: MessageFromWebControlDeleteCompetitorEvaluated = {
            type: "DeleteCompetitorEvaluated",
            data: dt,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendUpsertPDFSettingCommand(setting: PDFConfigurationSetting) {
        const packet: MessageFromWebControlStorePDFConfigurationSetting = {
            type: "StorePDFConfigurationSetting",
            data: setting,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendDeletePDFSettingCommand(id: Uuid) {
        const packet: MessageFromWebControlDeletePDFConfigurationSetting = {
            type: "DeletePDFConfigurationSetting",
            data: id,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendRequestAllPDFSettingsCommand() {
        const packet: MessageFromWebControlRequestPDFConfigurationSettings = {
            type: "RequestPDFConfigurationSettings",
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendRequestDevModeStatusCommand() {
        const packet: MessageFromWebControlRequestDevMode = {
            type: "RequestDevMode",
        };
        sendWSCommand(JSON.stringify(packet));
    }
    /**
     * @param ha id and heat_id are ignored, as they are set by the server
     */
    function sendCreateHeatAssignmentCommand(ha: HeatAssignment) {
        const packet: MessageFromWebControlCreateHeatAssignment = {
            type: "CreateHeatAssignment",
            data: ha,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendDeleteHeatAssignmentCommand(id: number) {
        const packet: MessageFromWebControlDeleteHeatAssignment = {
            type: "DeleteHeatAssignment",
            data: id,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendDebugDisplayCommand(entry: DisplayEntry) {
        const packet: MessageFromWebControlSendDebugDisplayCommand = {
            type: "SendDebugDisplayCommand",
            data: entry,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendGetHeatsCommand() {
        const packet: MessageFromWebControlGetHeats = {
            type: "GetHeats",
        };
        sendWSCommand(JSON.stringify(packet));
    }
    function sendGetMainHeatCommand() {
        const packet: MessageFromWebControlGetMainHeat = {
            type: "GetMainHeat",
        };
        sendWSCommand(JSON.stringify(packet));
    }
    const heatsMetaResult = ref([] as HeatMeta[]);
    function sendRequestTimingSettingsCommand() {
        const packet: MessageFromWebControlRequestTimingSettings = {
            type: "RequestTimingSettings",
        };
        sendWSCommand(JSON.stringify(packet));
    }

    function sendGetLogsCommand(how_many: number) {
        if (how_many < 0) {
            how_many = 1;
        }
        how_many = Math.floor(how_many);

        const packet: MessageFromWebControlGetLogs = {
            type: "GetLogs",
            data: how_many,
        };
        sendWSCommand(JSON.stringify(packet));
    }
    const logEntries = ref([] as PermanentlyStoredDataset[]);

    function sendSelectHeatCommand(id: string) {
        selectHeatId = id;
        sendSelectHeatCommandInternal();
    }
    function sendSelectHeatCommandInternal() {
        if (selectHeatId) {
            const packet: MessageFromWebControlSelectHeat = {
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
                const packet: MessageFromWebControlUpdateTimingSettings = {
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

    function sendGenericWSCommand(comm: MessageFromWebControl) {
        sendWSCommand(JSON.stringify(comm));
    }

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
            const windVal: RaceWind | null = parsed["wind"];

            if (windVal) {
                windValue.value = windStringRepr(windVal);
            }
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
        sendStaticallyConfigureServerCommand,
        sendExportToFileCommand,
        sendRequestAthletesCommand,
        sendUpsertAthleteCommand,
        sendDeleteAthleteCommand,
        sendCreateHeatAssignmentCommand,
        sendDeleteHeatAssignmentCommand,
        sendUpsertPDFSettingCommand,
        sendDeletePDFSettingCommand,
        sendGetMainHeatCommand,
        sendDeleteCompetitorEvaluatedCommand,
        sendDebugDisplayCommand,
        sendRequestDevModeStatusCommand,
        sendGenericWSCommand,
        canEditTimingSettings,
        timingSettings,
        selectedHeat,
        logEntries,
        heatsMetaResult,
        mainHeat,
        displayConnected,
        displayExternalPassthrough,
        displayCanSwitchMode,
        windServerConnected,
        windTime,
        windValue,
        requestedWindMeasurements,
        currentClientFrame,
        staticConfiguration,
        athletesData,
        pdfConfigurationSettings,
        versionMismatchTriggered,
        devMode,
        devMainHeatStartList,
        frametimeReport,
    };
});
