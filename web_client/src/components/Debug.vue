<template>
    <h2>Debug</h2>

    <div class="d-flex flex-column">
        <v-btn @click="sendMeta" class="mt-2" max-width="40em" :disabled="mainStore.devMainHeatStartList == null">
            Send Meta
        </v-btn>
        <v-btn @click="resetClock" class="mt-2" max-width="40em"> Reset </v-btn>
        <v-btn @click="startClock" class="mt-2" max-width="40em"> Start </v-btn>
        <v-btn @click="endSignal" class="mt-2" max-width="40em" :disabled="startTime == null"> End Signal </v-btn>
        <v-btn @click="debugDisplay" class="mt-2" max-width="40em"> Send Bib Test </v-btn>
    </div>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import {
        MessageFromWebControlDevRequestMainHeatStartList,
        MessageFromWebControlDevReset,
        MessageFromWebControlDevSendFinishSignal,
        MessageFromWebControlDevSendStartList,
        MessageFromWebControlDevStartRace,
    } from "../generated/interface";
    import { raceTimeFromNumber } from "../functions/representation";
    import { ref } from "vue";
    const mainStore = useMainStore();

    // on load, request a fake heat start list
    mainStore.sendGenericWSCommand({ type: "DevRequestMainHeatStartList" } as MessageFromWebControlDevRequestMainHeatStartList);

    function sendMeta() {
        if (mainStore.devMainHeatStartList != null) {
            mainStore.sendGenericWSCommand({
                type: "DevSendStartList",
                data: mainStore.devMainHeatStartList,
            } as MessageFromWebControlDevSendStartList);
        }
    }

    function resetClock() {
        mainStore.sendGenericWSCommand({ type: "DevReset" } as MessageFromWebControlDevReset);
    }

    const startTime = ref(null as null | number);

    function startClock() {
        mainStore.sendGenericWSCommand({ type: "DevStartRace" } as MessageFromWebControlDevStartRace);
        startTime.value = Date.now();
    }

    function endSignal() {
        if (startTime.value != null) {
            const time = (Date.now() - startTime.value) / 1000;

            mainStore.sendGenericWSCommand({
                type: "DevSendFinishSignal",
                data: raceTimeFromNumber(time),
            } as MessageFromWebControlDevSendFinishSignal);
        } else {
            console.error("None started... Sad");
        }
    }

    function debugDisplay() {
        function generateRandomPerson(): {
            firstName: string;
            lastName: string;
            number: number;
        } {
            const firstNames = ["Alice", "Bob", "Charlie", "Diana", "Ethan", "Fiona", "George", "Hannah", "Ivan", "Julia"];

            const lastNames = ["Smith", "Johnson", "Brown", "Taylor", "Anderson", "Clark", "Lewis", "Walker", "Hall", "Young"];

            const random = (max: number) => Math.floor(Math.random() * max);

            return {
                firstName: firstNames[random(firstNames.length)],
                lastName: lastNames[random(lastNames.length)],
                number: Math.floor(Math.random() * 900) + 100,
            };
        }

        const person = generateRandomPerson();
        mainStore.sendDebugDisplayCommand({
            bib: person.number,
            max_rounds: 4,
            name: person.firstName + " " + person.lastName,
            round: 1,
        });
    }
</script>

<style scoped></style>
