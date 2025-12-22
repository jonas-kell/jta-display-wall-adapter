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
    <div class="ma-2">
        <template v-if="!mainStore.connected && !connectedOnce">
            <h1>JTA Display Wall Adapter</h1>
            Connecting....
        </template>
        <template v-else>
            <template v-if="mainStore.staticConfiguration == null">
                <h1>JTA Display Wall Adapter</h1>
                <div class="ma-5">
                    <p class="mb-5">Server not Configured!!</p>

                    <v-text-field type="date" v-model="appDate" density="compact" label="Date this database will be used" />
                    <v-select
                        v-model="modeSelect"
                        density="compact"
                        label="Mode"
                        item-title="label"
                        item-value="value"
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

                    <br />
                    <v-btn @click="configure">Configure!</v-btn>
                </div>
            </template>
            <RouterView v-else />
        </template>
    </div>
</template>

<style scoped></style>
