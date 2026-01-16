<script setup lang="ts">
    import { ref, watch } from "vue";
    import useMainStore from "./stores/main";
    import useAuthStore from "./stores/auth";
    import Logs from "./components/Logs.vue";
    import ConnectionState from "./components/ConnectionState.vue";
    import TimingButtons from "./components/TimingButtons.vue";
    import InitDB from "./components/InitDB.vue";
    import { TODAY } from "./functions/date";
    import { ApplicationMode } from "./generated/interface";
    const mainStore = useMainStore();
    const authStore = useAuthStore();

    const connectedOnce = ref(false);
    watch(
        () => mainStore.connected,
        () => {
            if (mainStore.connected) {
                connectedOnce.value = true;
            }
        }
    );
    const fullyConnectedOnce = ref(false);
    watch(
        () => [connectedOnce.value, mainStore.connected, mainStore.staticConfiguration, authStore.password],
        () => {
            if (mainStore.connected && authStore.password != null && mainStore.staticConfiguration != null) {
                fullyConnectedOnce.value = true;
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

    const BOTTOM_EXPANDED_STORAGE_KEY = "BOTTOM_EXPANDED_STORAGE_KEY";
    const bottomExpanded = ref((localStorage.getItem(BOTTOM_EXPANDED_STORAGE_KEY) ?? "false") == "true");
    const toggle = () => {
        bottomExpanded.value = !bottomExpanded.value;
    };
    watch(bottomExpanded, () => {
        localStorage.setItem(BOTTOM_EXPANDED_STORAGE_KEY, String(bottomExpanded.value));
    });
</script>

<template>
    <template v-if="!fullyConnectedOnce || (!mainStore.connected && !connectedOnce)">
        <div class="ma-2">
            <h1>JTA Display Wall Adapter</h1>
            Connecting....
        </div>
    </template>
    <template v-else>
        <template v-if="!authStore.authenticated">
            <v-container fluid class="flex-column flex-grow-1 fill-height">
                <v-sheet color="surface-light" rounded="lg" class="flex-grow-1 w-100 pa-3">
                    <RouterView />
                </v-sheet>
            </v-container>
        </template>
        <template v-else>
            <template v-if="mainStore.staticConfiguration == null">
                <div class="ma-2">
                    <InitDB></InitDB>
                </div>
            </template>
            <v-app :class="{ 'v-theme--light': darkMode, 'v-theme--dark': darkMode }" :theme="darkMode ? 'dark' : 'light'" v-else>
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
                            {{ mainStore.staticConfiguration.mode }} at
                            <span>
                                <v-tooltip
                                    text="!!No changes are written to database, as database date does not match current date!!"
                                    location="bottom center"
                                    :disabled="mainStore.staticConfiguration.date == TODAY"
                                >
                                    <template v-slot:activator="{ props }">
                                        <span
                                            v-bind="props"
                                            :style="{
                                                color: mainStore.staticConfiguration.date == TODAY ? undefined : 'crimson',
                                            }"
                                            >{{ mainStore.staticConfiguration.date }}</span
                                        >
                                    </template>
                                </v-tooltip>
                            </span>
                            in {{ mainStore.staticConfiguration.meet_city }},
                            {{ mainStore.staticConfiguration.meet_location }}</span
                        >
                        <v-switch inset color="primary" v-model="darkMode" hide-details class="mr-5">
                            <template v-slot:label>
                                <v-icon :icon="darkMode ? 'mdi-weather-night' : 'mdi-weather-sunny'"></v-icon>
                            </template>
                        </v-switch>
                    </div>
                </v-app-bar>

                <v-navigation-drawer location="left" v-model="leftBar" :permanent="true">
                    <v-list-item>
                        <router-link to="/" class="router-link-style">Index</router-link>
                    </v-list-item>
                    <v-list-item>
                        <router-link to="/heats" class="router-link-style">Heats</router-link>
                    </v-list-item>
                    <v-list-item>
                        <router-link to="/timing" class="router-link-style">Timing</router-link>
                    </v-list-item>
                    <v-list-item
                        v-if="
                            mainStore.staticConfiguration.mode == ApplicationMode.SprinterKing ||
                            mainStore.staticConfiguration.mode == ApplicationMode.StreetLongRun
                        "
                    >
                        <router-link to="/manage_athletes" class="router-link-style">Manage Athletes</router-link>
                    </v-list-item>
                    <v-list-item>
                        <router-link to="/wind_request" class="router-link-style">Wind Request</router-link>
                    </v-list-item>
                    <v-list-item v-if="mainStore.staticConfiguration.mode == ApplicationMode.StreetLongRun">
                        <router-link to="/pdf_settings" class="router-link-style">PDF Settings</router-link>
                    </v-list-item>
                    <v-list-item v-if="mainStore.staticConfiguration.mode == ApplicationMode.StreetLongRun">
                        <router-link to="/pdf_print" class="router-link-style">PDF Print</router-link>
                    </v-list-item>
                    <v-list-item>
                        <router-link to="/bib_interface" class="router-link-style">Bib Interface</router-link>
                    </v-list-item>
                    <v-list-item v-if="mainStore.devMode">
                        <router-link to="/debug" class="router-link-style">DEBUG</router-link>
                    </v-list-item>
                    <v-divider></v-divider>
                    <div class="ma-2" v-if="mainStore.displayConnected">
                        <TimingButtons :has-free-text="false"></TimingButtons>
                    </div>
                </v-navigation-drawer>

                <v-navigation-drawer location="right" :permanent="false" :temporary="true" v-model="logs" width="600">
                    <Logs></Logs>
                </v-navigation-drawer>

                <v-navigation-drawer
                    location="bottom"
                    :permanent="true"
                    :width="bottomExpanded ? 200 : 60"
                    rounded="t-lg"
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
                    <v-container fluid class="flex-column flex-grow-1 fill-height">
                        <v-sheet color="surface-light" rounded="lg" class="flex-grow-1 w-100 pa-3">
                            <RouterView />
                        </v-sheet>
                    </v-container>
                </v-main>
            </v-app>
        </template>
    </template>
</template>

<style scoped></style>
