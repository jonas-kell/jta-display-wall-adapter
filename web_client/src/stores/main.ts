import { defineStore } from "pinia";
import { ref } from "vue";
import { WS_URL } from "../functions/environment";

export default defineStore("main", () => {
    const connected = ref(false);

    let ws = new WebSocket(WS_URL);

    ws.onmessage = (ev) => console.log(ev.data);

    function sendAdvertisementsCommand() {
        ws.send("TEST");
    }

    return { connected, sendAdvertisementsCommand };
});
