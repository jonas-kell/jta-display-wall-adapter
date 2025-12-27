<script setup lang="ts">
    import { ref } from "vue";
    import useMainStore from "./../stores/main";
    import { ApplicationMode } from "./../functions/interfaceShared";
    import { TODAY } from "../functions/date";
    import { uuid } from "../functions/uuid";

    const mainStore = useMainStore();

    const modeSelect = ref(ApplicationMode.TrackCompetition);

    const appDate = ref(TODAY);

    function configure() {
        mainStore.sendStaticallyConfigureServerCommand({
            date: appDate.value,
            mode: modeSelect.value,
            meet_id: uuid(),
            meet_city: appCity.value,
            meet_location: appLocation.value,
        });
    }

    const appCity = ref("");
    const appLocation = ref("");
</script>

<template>
    <h1>JTA Display Wall Adapter</h1>
    <div class="ma-5">
        <p class="mb-5">Server not Configured!!</p>

        <v-text-field
            type="date"
            v-model="appDate"
            density="compact"
            label="Date this database will be used"
            hide-details="auto"
        />
        <v-select
            v-model="modeSelect"
            density="compact"
            label="Mode"
            item-title="label"
            item-value="value"
            hide-details="auto"
            :items="[
                {
                    label: 'Normal Track Competition',
                    value: ApplicationMode.TrackCompetition,
                },
                {
                    label: 'Street Long Run',
                    value: ApplicationMode.StreetLongRun,
                },
                {
                    label: 'Sprinter König',
                    value: ApplicationMode.SprinterKing,
                },
            ]"
        >
        </v-select>
        <v-text-field v-model="appCity" density="compact" label="City where the meeting takes place" hide-details="auto" />
        <v-text-field
            v-model="appLocation"
            density="compact"
            label="Location where the meeting takes place"
            hide-details="auto"
        />

        <br />
        <v-btn @click="configure">Configure!</v-btn>
    </div>
</template>

<style scoped></style>
