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
    <button @click="mainStore.sendGetHeatStartsCommand">Get Heats</button>
    {{ mainStore.heatStartsResult }}
</template>

<script setup lang="ts">
    import { ref } from "vue";
    import useMainStore from "../stores/main";

    const freetext = ref("");

    const mainStore = useMainStore();
</script>

<style scoped></style>
