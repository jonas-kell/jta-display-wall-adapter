<script setup lang="ts">
    import { ref, watch } from "vue";
    import useMainStore from "./stores/main";
    import { ApplicationMode } from "./functions/interfaceShared";
    import { v4 as uuid } from "uuid";
    import Logs from "./components/Logs.vue";
    import ConnectionState from "./components/ConnectionState.vue";
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

    const leftBar = ref(true);
    const logs = ref(false);

    const DARK_MODE_STORAGE_KEY = "DARK_MODE_STORAGE_KEY";
    const darkMode = ref((localStorage.getItem(DARK_MODE_STORAGE_KEY) ?? "true") == "true");
    watch(darkMode, () => {
        localStorage.setItem(DARK_MODE_STORAGE_KEY, String(darkMode.value));
    });

    const bottomExpanded = ref(false);
    const toggle = () => {
        bottomExpanded.value = !bottomExpanded.value;
    };
</script>

<template>
    <template v-if="!mainStore.connected && !connectedOnce">
        <div class="ma-2">
            <h1>JTA Display Wall Adapter</h1>
            Connecting....
        </div>
    </template>
    <template v-else>
        <template v-if="mainStore.staticConfiguration == null">
            <div class="ma-2">
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
            </div>
        </template>
        <v-app
            :class="{ 'v-theme--light': darkMode, 'v-theme--dark': darkMode }"
            :theme="darkMode ? 'dark' : 'light'"
            class="rounded rounded-md border"
            v-else
        >
            <v-app-bar color="surface">
                <template v-slot:prepend>
                    <v-app-bar-nav-icon @click="leftBar = !leftBar"></v-app-bar-nav-icon>
                </template>
                <template v-slot:append>
                    <v-tooltip text="Logs">
                        <template v-slot:activator="{ props }">
                            <v-btn v-bind="props" icon="mdi-timeline-text" @click="logs = !logs"> </v-btn>
                        </template>
                    </v-tooltip>
                </template>
                <div class="d-flex flex-grow-1 justify-space-between align-center">
                    <h2>JTA Display Wall Adapter</h2>
                    <span v-if="mainStore.staticConfiguration != null">
                        {{ mainStore.staticConfiguration.mode }} at {{ mainStore.staticConfiguration.date }}</span
                    >
                    <v-switch inset color="primary" v-model="darkMode" hide-details class="mr-5">
                        <template v-slot:label>
                            <v-icon :icon="darkMode ? 'mdi-weather-night' : 'mdi-weather-sunny'"></v-icon>
                        </template>
                    </v-switch>
                </div>
            </v-app-bar>

            <v-navigation-drawer location="left" v-model="leftBar" :permanent="true"> </v-navigation-drawer>

            <v-navigation-drawer location="right" :permanent="false" :temporary="true" v-model="logs" width="600">
                <Logs></Logs>
            </v-navigation-drawer>

            <v-navigation-drawer
                location="bottom"
                :permanent="true"
                :width="bottomExpanded ? 180 : 60"
                rounded="lg"
                class="px-4 py-1"
            >
                <div class="w-100 fill-height d-flex flex-row">
                    <v-container fluid class="pa-2 flex-grow-1 d-flex">
                        <ConnectionState
                            :collapsed="!bottomExpanded"
                            :class="bottomExpanded ? 'align-self-start' : 'align-self-center'"
                        ></ConnectionState>
                    </v-container>
                    <v-btn
                        icon
                        @click="toggle"
                        density="comfortable"
                        variant="tonal"
                        :class="bottomExpanded ? 'align-self-start' : 'align-self-center'"
                    >
                        <v-icon>
                            {{ bottomExpanded ? "mdi-chevron-down" : "mdi-chevron-up" }}
                        </v-icon>
                    </v-btn>
                </div>
            </v-navigation-drawer>

            <v-main class="d-flex">
                <v-container fluid class="flex-column flex-grow-1 fill-height pa-2">
                    <v-sheet color="surface-light" rounded="lg" class="flex-grow-1 w-100 pa-3">
                        <RouterView />
                    </v-sheet>
                </v-container>
            </v-main>
        </v-app>
    </template>
</template>

<style scoped></style>
