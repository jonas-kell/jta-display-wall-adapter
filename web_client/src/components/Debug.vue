<template>
    <h2>Debug</h2>

    <div class="d-flex flex-column">
        <v-btn @click="sendMeta" class="mt-2" max-width="40em" :disabled="mainStore.devMainHeatStartList == null">
            Send Meta
        </v-btn>
        <v-btn @click="resetClock" class="mt-2" max-width="40em"> Reset </v-btn>
        <v-btn @click="startClock" class="mt-2" max-width="40em"> Start </v-btn>
        <v-btn @click="endSignal" class="mt-2" max-width="40em" :disabled="startTime == null"> End Signal </v-btn>
        <v-btn @click="evaluateOneAthlete" class="mt-2" max-width="40em" :disabled="mainStore.devMainHeatStartList == null">
            Evaluate one Athlete
        </v-btn>
        <v-btn @click="debugDisplay" class="mt-2" max-width="40em"> Send Bib Test </v-btn>
    </div>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import {
        HeatCompetitor,
        HeatCompetitorResult,
        MessageFromWebControlDevRequestMainHeatStartList,
        MessageFromWebControlDevReset,
        MessageFromWebControlDevSendEvaluated,
        MessageFromWebControlDevSendFinishSignal,
        MessageFromWebControlDevSendResultList,
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
    const finishIndex = ref(-1);
    const times = ref([] as number[]);

    function startClock() {
        mainStore.sendGenericWSCommand({ type: "DevStartRace" } as MessageFromWebControlDevStartRace);
        startTime.value = Date.now();
        finishIndex.value = -1;
        times.value = [];
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

    function getAthleteAtIndex(): HeatCompetitor {
        if (mainStore.devMainHeatStartList == null || mainStore.devMainHeatStartList.competitors.length == 0) {
            return {
                bib: 0,
                class: "X",
                club: "Club",
                disqualified: null,
                first_name: "Firstname",
                last_name: "Lastname",
                gender: "X",
                id: "0",
                lane: 1,
                nation: "GER",
            };
        } else {
            if (finishIndex.value < mainStore.devMainHeatStartList.competitors.length - 1) {
                finishIndex.value += 1;
            }

            const comp = mainStore.devMainHeatStartList.competitors[finishIndex.value];
            return comp;
        }
    }

    function evaluateOneAthlete() {
        if (mainStore.devMainHeatStartList != null && startTime.value != null) {
            const heat = mainStore.devMainHeatStartList;
            const heatId = heat.id;
            const last = getAthleteAtIndex();

            const time = (Date.now() - startTime.value) / 1000;
            if (times.value.length < heat.competitors.length) {
                times.value.push(time);
            }
            const lastTime = times.value[times.value.length - 1];

            mainStore.sendGenericWSCommand({
                type: "DevSendEvaluated",
                data: {
                    application: "DevTest",
                    id: heatId,
                    version: "0.0.0",
                    generated: "2026-01-01T10:10:10",
                    competitor_result: {
                        competitor: last,
                        difference_to_previous: { type: "Winner" }, // this is not really used, we just send fake data here
                        difference_to_winner: { type: "Winner" }, // this is not really used, we just send fake data here
                        distance: heat.distance_meters,
                        finish_time: {
                            // this is not really used, we just send fake data here
                            hours: 10,
                            minutes: 10,
                            seconds: 10,
                            fractional_part_in_ten_thousands: null,
                        },
                        rank: 1, // this is not really used, we just send fake data here
                        runtime: raceTimeFromNumber(lastTime),
                        runtime_full_precision: raceTimeFromNumber(lastTime),
                    },
                },
            } as MessageFromWebControlDevSendEvaluated);

            const results = times.value.map((time, index): HeatCompetitorResult => {
                const comp = heat.competitors[index];
                return {
                    competitor: comp,
                    difference_to_previous: { type: "Winner" }, // this is not really used, we just send fake data here
                    difference_to_winner: { type: "Winner" }, // this is not really used, we just send fake data here
                    distance: heat.distance_meters,
                    finish_time: {
                        // this is not really used, we just send fake data here
                        hours: 10,
                        minutes: 10,
                        seconds: 10,
                        fractional_part_in_ten_thousands: null,
                    },
                    rank: 1, // this is not really used, we just send fake data here
                    runtime: raceTimeFromNumber(time),
                    runtime_full_precision: raceTimeFromNumber(time),
                };
            });
            const leftToEval = heat.competitors.filter((comp) => {
                return !results.some((evaledComp) => {
                    return evaledComp.competitor.bib == comp.bib;
                });
            });

            mainStore.sendGenericWSCommand({
                type: "DevSendResultList",
                data: {
                    distance_meters: heat.distance_meters,
                    id: heatId,
                    name: heat.name,
                    wind: null, // TODO support wind!
                    start_time: {
                        // this is not really used, we just send fake data here
                        hours: 10,
                        minutes: 10,
                        seconds: 10,
                        fractional_part_in_ten_thousands: null,
                    },
                    competitors_evaluated: results,
                    competitors_left_to_evaluate: leftToEval,
                },
            } as MessageFromWebControlDevSendResultList);
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
