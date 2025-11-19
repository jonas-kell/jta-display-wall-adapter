import { defineStore } from "pinia";
import { ref } from "vue";
import { WS_URL } from "../functions/environment";
import { Advertisements, FreeText, Idle } from "../functions/interfaceOutbound";
import { InboundMessageType, parseMessage } from "../functions/interfaceInbound";

function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

export default defineStore("main", () => {
    const connected = ref(false);
    const displayConnected = ref(false);
    const displayExternalPassthrough = ref(false);

    let reconnecting = false;
    let ws = null as null | WebSocket;

    function handleWSMessage(ev: any) {
        let msg = parseMessage(JSON.parse(ev.data));

        switch (msg.type) {
            case InboundMessageType.DisplayClientState:
                displayConnected.value = msg.data.alive;
                displayExternalPassthrough.value = msg.data.external_passthrough_mode;
                break;
            case InboundMessageType.Unknown:
                console.error("Received unknown message type:", msg.data);
                break;
        }
    }

    async function initWS() {
        if (reconnecting) return; // prevent multiple runs
        reconnecting = true;
        console.log("Reconnecting WS");

        if (ws) {
            try {
                ws.close();
            } catch (_) {}
            ws = null;
            await sleep(500);
        }

        ws = new WebSocket(WS_URL);
        ws.onclose = () => {
            connected.value = false;
            initWS();
        };
        ws.onerror = async () => {
            connected.value = false;
            initWS();
        };
        ws.onmessage = handleWSMessage;
        ws.onopen = () => {
            connected.value = true;
            reconnecting = false;
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

    return {
        connected,
        sendAdvertisementsCommand,
        sendIdleCommand,
        sendFreetextCommand,
        displayConnected,
        displayExternalPassthrough,
    };
});
