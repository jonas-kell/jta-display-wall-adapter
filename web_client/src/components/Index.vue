<template>
    <h1>JTA Display Wall Adapter</h1>
    Connected: {{ mainStore.connected }} ({{ mainStore.staticConfiguration?.mode }} at {{ mainStore.staticConfiguration?.date }})
    <br />
    Display Connected: {{ mainStore.displayConnected }}, Mode:
    {{ mainStore.displayExternalPassthrough ? "External Passthrough" : "Default client" }}
    <button @click="mainStore.sendSwitchModeCommand" :disabled="!mainStore.displayCanSwitchMode">Switch Mode</button>
    <br />
    Wind Connected: {{ mainStore.windServerConnected }} {{ mainStore.windValue }}
    {{ mainStore.windServerConnected ? mainStore.windTime : "" }}
    <br />
    <br />

    <button :disabled="mainStore.displayExternalPassthrough" @click="mainStore.sendAdvertisementsCommand">Advertisements</button>
    <button :disabled="mainStore.displayExternalPassthrough" @click="mainStore.sendIdleCommand">Idle</button>
    <button :disabled="mainStore.displayExternalPassthrough" @click="mainStore.sendClockCommand">Clock</button>
    <button :disabled="mainStore.displayExternalPassthrough" @click="mainStore.sendStartListCommand">Start List</button>
    <button :disabled="mainStore.displayExternalPassthrough" @click="mainStore.sendTimingCommand">Timing</button>
    <button :disabled="mainStore.displayExternalPassthrough" @click="mainStore.sendResultListCommand">Result List</button>
    <br />
    <br />
    Timing Settings:
    <br />
    <b style="color: crimson">Caution!! For full functionality, Camera Program Display Wall Mode MUST be on: AUTO</b>
    <br />
    <b style="color: crimson">To avoid unintended skipping, Timing Program Auto-Display must be: false</b>
    <br />
    Otherwise only the first light barrier signal is forwarded as a HeatFinish
    <br />
    If the button <b>Anzeigetafel</b> is red and mode is fixed and can not be changed, you can click the title (red) to enable
    editing the wall state again
    <br />
    <template v-if="mainStore.timingSettings">
        <div style="margin-left: 1cm">
            <p>
                Intermediate Fireworks: {{ mainStore.timingSettings.fireworks_on_intermediate ? "yes" : "no" }}
                <button
                    @click="
                        mainStore.timingSettings.fireworks_on_intermediate = !mainStore.timingSettings.fireworks_on_intermediate
                    "
                >
                    Toggle
                </button>
            </p>
            <p>
                Finish Fireworks: {{ mainStore.timingSettings.fireworks_on_finish ? "yes" : "no" }}
                <button @click="mainStore.timingSettings.fireworks_on_finish = !mainStore.timingSettings.fireworks_on_finish">
                    Toggle
                </button>
            </p>
            <p>
                Timing decimal places:
                <input
                    type="number"
                    min="-1"
                    max="4"
                    v-model="mainStore.timingSettings.max_decimal_places_after_comma"
                    step="1"
                />
            </p>
            <p>
                Hold time ms:
                <input type="number" min="0" max="15000" v-model="mainStore.timingSettings.hold_time_ms" step="100" />
            </p>
            <p>
                Start Sound: {{ mainStore.timingSettings.play_sound_on_start ? "yes" : "no" }}
                <button @click="mainStore.timingSettings.play_sound_on_start = !mainStore.timingSettings.play_sound_on_start">
                    Toggle
                </button>
            </p>
            <p>
                Intermediate Sound: {{ mainStore.timingSettings.play_sound_on_intermediate ? "yes" : "no" }}
                <button
                    @click="
                        mainStore.timingSettings.play_sound_on_intermediate = !mainStore.timingSettings.play_sound_on_intermediate
                    "
                >
                    Toggle
                </button>
            </p>
            <p>
                Finish Sound: {{ mainStore.timingSettings.play_sound_on_finish ? "yes" : "no" }}
                <button @click="mainStore.timingSettings.play_sound_on_finish = !mainStore.timingSettings.play_sound_on_finish">
                    Toggle
                </button>
            </p>
            <p>
                Meta can change: {{ mainStore.timingSettings.can_currently_update_meta ? "yes" : "no" }}
                <button
                    @click="
                        mainStore.timingSettings.can_currently_update_meta = !mainStore.timingSettings.can_currently_update_meta
                    "
                >
                    Toggle
                </button>
            </p>
            <p>
                Time continues running: {{ mainStore.timingSettings.time_continues_running ? "yes" : "no" }}
                <button
                    @click="mainStore.timingSettings.time_continues_running = !mainStore.timingSettings.time_continues_running"
                >
                    Toggle
                </button>
            </p>
            <p>
                Switch to Start List automatically:
                {{ mainStore.timingSettings.switch_to_start_list_automatically ? "yes" : "no" }}
                <button
                    @click="
                        mainStore.timingSettings.switch_to_start_list_automatically =
                            !mainStore.timingSettings.switch_to_start_list_automatically
                    "
                >
                    Toggle
                </button>
            </p>
            <p>
                Switch to timing automatically: {{ mainStore.timingSettings.switch_to_timing_automatically ? "yes" : "no" }}
                <button
                    @click="
                        mainStore.timingSettings.switch_to_timing_automatically =
                            !mainStore.timingSettings.switch_to_timing_automatically
                    "
                >
                    Toggle
                </button>
            </p>
            <p>
                Switch to results automatically: {{ mainStore.timingSettings.switch_to_results_automatically ? "yes" : "no" }}
                <button
                    @click="
                        mainStore.timingSettings.switch_to_results_automatically =
                            !mainStore.timingSettings.switch_to_results_automatically
                    "
                >
                    Toggle
                </button>
            </p>
        </div>
    </template>
    <p v-else>Not loaded</p>
    <br />

    Client Display<span v-if="mainStore.displayExternalPassthrough"> (From External)</span>:
    <br />
    <template v-if="mainStore.displayConnected">
        <img width="360px" v-if="mainStore.currentClientFrame" :src="mainStore.currentClientFrame" />
    </template>
    <p v-else>No Connection</p>
    <br />
    <br />

    <textarea v-model="freetext" :disabled="mainStore.displayExternalPassthrough" @keydown="checkFreetextSubmit"></textarea>
    <br />
    <button :disabled="mainStore.displayExternalPassthrough" @click="mainStore.sendFreetextCommand(freetext.trim())">
        Send Freetext
    </button>
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

    let lastWasEnter = false;
    function checkFreetextSubmit(event: KeyboardEvent) {
        if (event.code == "Enter") {
            if (lastWasEnter) {
                lastWasEnter = false;
                freetext.value = freetext.value.trim();
                mainStore.sendFreetextCommand(freetext.value.trim());
                return;
            } else {
                lastWasEnter = true;
            }
        } else {
            lastWasEnter = false;
        }
        if (event.ctrlKey && event.code == "Enter") {
            lastWasEnter = false;
            freetext.value = freetext.value.trim();
            mainStore.sendFreetextCommand(freetext.value.trim());
        }
    }
</script>

<style scoped></style>
