<template>
    <div class="w-100 d-flex flex-row justify-space-between">
        <div class="d-flex flex-column">
            <div
                class="d-flex flex-wrap"
                :class="props.collapsed ? 'flex-row' : 'flex-column'"
                :style="{ 'max-height': '130px' }"
            >
                <span class="py-1 px-2">
                    Server:

                    <v-tooltip
                        text="!!No changes are written to database, as database date does not match current date!!"
                        location="top center"
                        :disabled="rightDate"
                    >
                        <template v-slot:activator="{ props }">
                            <v-icon
                                v-bind="props"
                                icon="mdi-circle"
                                :color="mainStore.connected ? (rightDate ? 'green' : 'yellow') : 'red'"
                            ></v-icon>
                        </template>
                    </v-tooltip>
                </span>
                <template v-if="mainStore.connectionState != null">
                    <span class="py-1 px-2" v-if="mainStore.connectionState.try_conect_to_display_client">
                        Display:
                        <v-tooltip :text="mainStore.connectionState.display_client_address_with_port" location="top center">
                            <template v-slot:activator="{ props }">
                                <v-icon
                                    v-bind="props"
                                    icon="mdi-circle"
                                    :color="mainStore.displayConnected ? 'green' : 'red'"
                                ></v-icon>
                            </template>
                        </v-tooltip>
                        (Mode: {{ mainStore.displayExternalPassthrough ? "Ext. Passth." : "Default" }})
                    </span>
                    <span class="py-1 px-2" v-if="mainStore.connectionState.try_connect_to_wind">
                        Wind:
                        <v-tooltip :text="mainStore.connectionState.wind_address_with_port" location="top center">
                            <template v-slot:activator="{ props }">
                                <v-icon
                                    v-bind="props"
                                    icon="mdi-circle"
                                    :color="
                                        mainStore.windServerLive
                                            ? 'green'
                                            : mainStore.connectionState.wind_connected
                                              ? 'yellow'
                                              : 'red'
                                    "
                                ></v-icon>
                            </template>
                        </v-tooltip>
                        {{ mainStore.windValue }}
                        {{ mainStore.windServerLive ? mainStore.windTime : "" }}
                    </span>
                    <span class="py-1 px-2" v-if="mainStore.connectionState.try_to_connect_to_camera_program">
                        Cam prog.:
                        <v-tooltip
                            :text="
                                mainStore.connectionState.camera_program_address +
                                ': ' +
                                'timing: ' +
                                mainStore.connectionState.camera_program_timing_port +
                                (mainStore.connectionState.camera_program_connected_on_timing_port ? ' (✓)' : ' (x)') +
                                ', data: ' +
                                mainStore.connectionState.camera_program_data_port +
                                (mainStore.connectionState.camera_program_connected_on_data_port ? ' (✓)' : ' (x)') +
                                ', xml: ' +
                                mainStore.connectionState.camera_program_xml_port +
                                (mainStore.connectionState.camera_program_connected_on_xml_port ? ' (✓)' : ' (x)')
                            "
                            location="top center"
                        >
                            <template v-slot:activator="{ props }">
                                <v-icon
                                    v-bind="props"
                                    icon="mdi-circle"
                                    :color="mainStore.connectionState.camera_program_connected ? 'green' : 'red'"
                                ></v-icon>
                            </template>
                        </v-tooltip>
                    </span>
                    <span class="py-1 px-2" v-if="mainStore.connectionState.try_connect_to_bib">
                        Bib:
                        <v-tooltip :text="mainStore.connectionState.bib_address_with_port" location="top center">
                            <template v-slot:activator="{ props }">
                                <v-icon
                                    v-bind="props"
                                    icon="mdi-circle"
                                    :color="mainStore.connectionState.bib_connected ? 'green' : 'red'"
                                ></v-icon>
                            </template>
                        </v-tooltip>
                    </span>
                    <span class="py-1 px-2" v-if="mainStore.connectionState.listening_to_timing_program">
                        Timing prog.:
                        <v-icon
                            icon="mdi-circle"
                            :color="mainStore.connectionState.timing_program_is_connected ? 'green' : 'red'"
                        ></v-icon>
                    </span>
                    <span class="py-1 px-2" v-if="mainStore.connectionState.try_to_connect_to_display_passthrough">
                        Disp. PT:
                        <v-tooltip :text="mainStore.connectionState.display_passthrough_address" location="top center">
                            <template v-slot:activator="{ props }">
                                <v-icon
                                    v-bind="props"
                                    icon="mdi-circle"
                                    :color="mainStore.connectionState.display_passthrough_connected ? 'green' : 'red'"
                                ></v-icon>
                            </template>
                        </v-tooltip>
                    </span>
                    <span class="py-1 px-2" v-if="mainStore.connectionState.try_to_connect_to_idcapture">
                        ID capt.:
                        <v-tooltip :text="mainStore.connectionState.idcapture_address_with_port" location="top center">
                            <template v-slot:activator="{ props }">
                                <v-icon
                                    v-bind="props"
                                    icon="mdi-circle"
                                    :color="mainStore.connectionState.idcapture_connected ? 'green' : 'red'"
                                ></v-icon>
                            </template>
                        </v-tooltip>
                    </span>
                </template>
            </div>

            <v-btn
                @click="mainStore.sendSwitchModeCommand"
                :disabled="!mainStore.displayCanSwitchMode"
                density="comfortable"
                class="ma-2"
                v-if="!props.collapsed"
                variant="tonal"
            >
                Switch Display Mode
            </v-btn>
        </div>
        <div v-if="!props.collapsed" class="d-flex flex-column mx-5">
            <div>
                Client Display<span v-if="mainStore.displayExternalPassthrough"> (From External)</span>:
                <br />
                <template v-if="mainStore.displayConnected">
                    <img
                        width="360px"
                        style="max-height: 120px"
                        v-if="mainStore.currentClientFrame"
                        :src="mainStore.currentClientFrame"
                    />
                    <br />

                    <v-tooltip
                        :text="'top frame times: ' + mainStore.frametimeReport.worst_n.map((a) => String(a) + '%').join(' - ')"
                        location="top center"
                        v-if="mainStore.frametimeReport"
                    >
                        <template v-slot:activator="{ props }">
                            <span v-bind="props">
                                (render takes {{ mainStore.frametimeReport.time_percentage_taken_per_frame_since_last_report }}%
                                of time to reach {{ mainStore.frametimeReport.target_fps }}FPS )</span
                            >

                            <span style="color: crimson" v-if="max > 100"> {{ max }}%</span>
                        </template>
                    </v-tooltip>
                </template>
                <p v-else>No Connection</p>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
    import { computed } from "vue";
    import useMainStore from "../stores/main";
    import { TODAY } from "../functions/date";
    const mainStore = useMainStore();

    const props = defineProps<{ collapsed: boolean }>();

    const rightDate = computed(() => {
        return (mainStore.staticConfiguration?.date ?? "") == TODAY;
    });

    const max = computed(() => {
        if (mainStore.frametimeReport != null) {
            return Math.max(...mainStore.frametimeReport.worst_n);
        } else {
            return 0;
        }
    });
</script>

<style scoped></style>
