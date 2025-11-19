import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";
import { WS_URL } from "../functions/environment";
import { Advertisements, FreeText, Idle, RequestDisplayClientState, SwitchMode } from "../functions/interfaceOutbound";
import { InboundMessageType, parseMessage } from "../functions/interfaceInbound";

function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

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

    function handleWSMessage(ev: any) {
        let msg = parseMessage(JSON.parse(ev.data));

        switch (msg.type) {
            case InboundMessageType.DisplayClientState:
                displayConnected.value = msg.data.alive;
                displayExternalPassthrough.value = msg.data.external_passthrough_mode;
                displayCanSwitchModeInternal.value = msg.data.can_switch_mode;
                break;
            case InboundMessageType.Unknown:
                console.error("Received unknown message type:", msg.data);
                break;
        }
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

            console.log("Socket connceted");

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

    const displayCanSwitchMode = computed(() => {
        return displayCanSwitchModeInternal.value && displayConnected.value;
    });

    return {
        connected,
        sendSwitchModeCommand,
        sendAdvertisementsCommand,
        sendIdleCommand,
        sendFreetextCommand,
        displayConnected,
        displayExternalPassthrough,
        displayCanSwitchMode,
    };
});
