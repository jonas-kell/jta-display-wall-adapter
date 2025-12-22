<template>
    <div class="d-flex flex-column">
        <template v-if="props.hasFreeText">
            <div>
                <v-textarea
                    class="mt-4"
                    hide-details="auto"
                    rows="1"
                    max-rows="4"
                    auto-grow
                    v-model="freetext"
                    :disabled="mainStore.displayExternalPassthrough"
                    @keydown="checkFreetextSubmit"
                ></v-textarea>
            </div>
            <v-btn
                variant="tonal"
                density="compact"
                :disabled="mainStore.displayExternalPassthrough"
                @click="mainStore.sendFreetextCommand(freetext.trim())"
            >
                Send Freetext
            </v-btn>
        </template>
        <v-btn
            variant="tonal"
            density="compact"
            class="mt-1"
            :disabled="mainStore.displayExternalPassthrough"
            @click="mainStore.sendIdleCommand"
            >Idle</v-btn
        >
        <v-btn
            variant="tonal"
            density="compact"
            class="mt-1"
            :disabled="mainStore.displayExternalPassthrough"
            @click="mainStore.sendClockCommand"
            >Clock</v-btn
        >
        <v-btn
            variant="tonal"
            density="compact"
            class="mt-1"
            :disabled="mainStore.displayExternalPassthrough"
            @click="mainStore.sendAdvertisementsCommand"
        >
            Advertisements
        </v-btn>
        <v-btn
            variant="tonal"
            density="compact"
            class="mt-1"
            :disabled="mainStore.displayExternalPassthrough"
            @click="mainStore.sendStartListCommand"
            >Start List</v-btn
        >
        <v-btn
            variant="tonal"
            density="compact"
            class="mt-1"
            :disabled="mainStore.displayExternalPassthrough"
            @click="mainStore.sendTimingCommand"
            >Timing</v-btn
        >
        <v-btn
            variant="tonal"
            density="compact"
            class="mt-1"
            :disabled="mainStore.displayExternalPassthrough"
            @click="mainStore.sendResultListCommand"
            >Result List</v-btn
        >
    </div>
</template>

<script setup lang="ts">
    const props = defineProps<{ hasFreeText: boolean }>();

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
