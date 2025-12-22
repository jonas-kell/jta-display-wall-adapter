<template>
    <h2>Timing</h2>

    <div class="d-flex flex-row justify-space-between">
        <div>
            <div>
                <b style="color: crimson">Caution!!</b> For full functionality, Camera Program Display Wall Mode MUST be on:
                <b style="color: crimson">AUTO</b>
                <br />
                To avoid unintended skipping, Timing Program Auto-Display must be: <b style="color: crimson">false</b>
                <br />
                Otherwise only the first light barrier signal is forwarded as a HeatFinish
                <br />
                If the button <b>Anzeigetafel</b> is red and mode is fixed and can not be changed, you can click the title (red)
                to enable editing the wall state again
            </div>

            <div class="mt-5">
                <h3>Settings</h3>
                <template v-if="mainStore.timingSettings">
                    <div>
                        <v-text-field
                            density="compact"
                            type="number"
                            min="-1"
                            max="4"
                            v-model.number="mainStore.timingSettings.max_decimal_places_after_comma"
                            step="1"
                            label="Timing decimal places"
                            width="250"
                            hide-details
                            class="mt-1"
                        />
                        <v-text-field
                            class="mt-1"
                            density="compact"
                            type="number"
                            min="0"
                            max="15000"
                            v-model.number="mainStore.timingSettings.hold_time_ms"
                            step="100"
                            label="Hold time ms"
                            width="250"
                            hide-details
                        />
                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.fireworks_on_intermediate"
                            hide-details
                            label="Intermediate Fireworks"
                        >
                        </v-switch>
                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.fireworks_on_finish"
                            hide-details
                            label="Finish Fireworks"
                        >
                        </v-switch>
                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.play_sound_on_start"
                            hide-details
                            label="Start Sound"
                        >
                        </v-switch>
                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.play_sound_on_intermediate"
                            hide-details
                            label="Intermediate Sound"
                        >
                        </v-switch>
                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.play_sound_on_finish"
                            hide-details
                            label="Finish Sound"
                        >
                        </v-switch>
                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.can_currently_update_meta"
                            hide-details
                            label="Meta can change"
                        >
                        </v-switch>
                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.time_continues_running"
                            hide-details
                            label="Time continues running"
                        >
                        </v-switch>

                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.switch_to_start_list_automatically"
                            hide-details
                            label="Switch to Start List automatically"
                        >
                        </v-switch>
                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.switch_to_timing_automatically"
                            hide-details
                            label="Switch to Timing automatically"
                        >
                        </v-switch>
                        <v-switch
                            color="primary"
                            density="compact"
                            v-model="mainStore.timingSettings.switch_to_results_automatically"
                            hide-details
                            label="Switch to Results automatically"
                        >
                        </v-switch>
                    </div>
                </template>
                <p v-else>Not loaded</p>
            </div>
        </div>
        <TimingButtons :has-free-text="true"></TimingButtons>
    </div>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import TimingButtons from "./TimingButtons.vue";

    const mainStore = useMainStore();
</script>

<style scoped></style>
