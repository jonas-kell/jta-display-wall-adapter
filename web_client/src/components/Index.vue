<template>
    <h1>JTA Display Wall Adapter</h1>

    <button @click="mainStore.sendAdvertisementsCommand">Advertisements</button>
    <button @click="mainStore.sendIdleCommand">Idle</button>

    <br />
    <br />
    <input type="text" v-model="freetext" />
    <button @click="mainStore.sendFreetextCommand(freetext)">Freetext</button>
    <br />
    <br />
    Connected: {{ mainStore.connected }}
    <br />
    <br />
    Display Connected: {{ mainStore.displayConnected }}, Mode:
    {{ mainStore.displayExternalPassthrough ? "External Passthrough" : "Default client" }}
    <button @click="mainStore.sendSwitchModeCommand" :disabled="!mainStore.displayCanSwitchMode">Switch Mode</button>
    <br />
    <br />
    <button @click="mainStore.sendGetHeatsCommand">Get Heats</button>
    <p v-for="heatEntry in mainStore.heatsMetaResult">
        {{ heatEntry.name }}, Nr: {{ heatEntry.number }}, Time: {{ heatEntry.scheduled_start_time_string }}
        <button @click="mainStore.sendSelectHeatCommand(heatEntry.id)">Select</button>
    </p>
    <p v-if="mainStore.heatsMetaResult.length == 0">No heats loaded/available</p>
    <br />
    <br />
    <template v-if="mainStore.selectedHeat">
        Heat Selected: {{ mainStore.selectedHeat.meta.name }} <br />
        <pre>{{ mainStore.selectedHeat }}</pre>
    </template>
    <p v-else>No heat selected</p>
    <br />
    <br />
    <button @click="mainStore.sendGetLogsCommand(10000)">Get Logs</button>
    <p v-for="logEntry in mainStore.logEntries">
        Time: {{ logEntry.stored_at }}, Key: {{ logEntry.name_key }}, Data: {{ JSON.parse(logEntry.data) }}
    </p>
    <p v-if="mainStore.logEntries.length == 0">No logs loaded/available</p>
</template>

<script setup lang="ts">
    import { ref } from "vue";
    import useMainStore from "../stores/main";

    const freetext = ref("");

    const mainStore = useMainStore();
</script>

<style scoped></style>
