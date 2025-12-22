<script setup lang="ts">
    import { ref, watch } from "vue";
    import useMainStore from "./stores/main";
    import { ApplicationMode } from "./functions/interfaceShared";
    import { v4 as uuid } from "uuid";
    const mainStore = useMainStore();

    const modeSelect = ref(ApplicationMode.TrackCompetition);

    const today = new Date().toISOString().split("T")[0];
    const appDate = ref(today);

    function configure() {
        mainStore.sendStaticallyConfigureServerCommand({
            date: appDate.value,
            mode: modeSelect.value,
            meet_id: uuid(),
        });
    }

    const connectedOnce = ref(false);
    watch(
        () => mainStore.connected,
        () => {
            if (mainStore.connected) {
                connectedOnce.value = true;
            }
        }
    );
</script>

<template>
    <template v-if="!mainStore.connected && !connectedOnce">
        <h1>JTA Display Wall Adapter</h1>
        Connecting....
    </template>
    <template v-else>
        <template v-if="mainStore.staticConfiguration == null">
            <p>Server not Configured!!</p>
            <br />
            Date this database will be used:
            <br />
            <input type="date" v-model="appDate" />

            <br />
            <br />

            Mode:
            <br />
            <select v-model="modeSelect" style="min-width: 100px">
                <option :value="ApplicationMode.TrackCompetition">Normal Track Competition</option>
                <option :value="ApplicationMode.StreetLongRun">Street Long Run</option>
                <option :value="ApplicationMode.SprinterKing">Sprinter König</option>
            </select>

            <br />
            <br />
            <button @click="configure">Configure!</button>
        </template>
        <RouterView v-else />
    </template>
</template>

<style scoped></style>
