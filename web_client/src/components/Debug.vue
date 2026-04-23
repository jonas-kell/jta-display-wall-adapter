<template>
    <h2>Debug</h2>

    <div class="d-flex flex-column">
        <template v-if="mainStore.devMainHeatStartList != null">
            <span v-if="mainStore.devMainHeatStartList.competitors.length == 0">
                Load Competitors (Go street race mode and "Manage Athletes")</span
            >
            <span v-else> {{ mainStore.devMainHeatStartList.competitors.length }} Competitors Loaded</span>
        </template>
        <v-btn @click="sendMeta" class="mt-2" max-width="40em" :disabled="mainStore.devMainHeatStartList == null">
            Send Meta
        </v-btn>
        <v-btn @click="resetClock" class="mt-2" max-width="40em"> Reset </v-btn>
        <v-btn @click="startClock" class="mt-2" max-width="40em" :disabled="debugStore.startTime != null"> Start </v-btn>
        <v-btn @click="intermediateSignal" class="mt-2" max-width="40em" :disabled="debugStore.startTime == null">
            Intermediate Signal
        </v-btn>
        <v-btn @click="endSignal" class="mt-2" max-width="40em" :disabled="debugStore.startTime == null"> End Signal </v-btn>
        <v-btn @click="sendWind" class="mt-2" max-width="40em" :disabled="debugStore.startTime == null"> Wind </v-btn>
        <v-btn
            @click="evaluateOneAthlete"
            class="mt-2"
            max-width="40em"
            :disabled="debugStore.startTime == null || mainStore.devMainHeatStartList == null"
        >
            Evaluate one Athlete
        </v-btn>
        <v-btn @click="debugDisplay" class="mt-2" max-width="40em"> Send Bib Test </v-btn>
    </div>
</template>

<script setup lang="ts">
    // TODO this page only works if athletes are loaded in MainHeat -> convert to support all heats
    import useMainStore from "../stores/main";
    import useDebugStore from "../stores/debug";
    import {
        DayTime,
        HeatCompetitor,
        HeatCompetitorResult,
        MessageFromWebControlDevRequestMainHeatStartList,
        MessageFromWebControlDevReset,
        MessageFromWebControlDevSendEvaluated,
        MessageFromWebControlDevSendFinishSignal,
        MessageFromWebControlDevSendIntermediateSignal,
        MessageFromWebControlDevSendResultList,
        MessageFromWebControlDevSendStartList,
        MessageFromWebControlDevSendWind,
        MessageFromWebControlDevStartRace,
    } from "../generated/interface";
    import { raceTimeFromNumber } from "../functions/representation";
    const mainStore = useMainStore();
    const debugStore = useDebugStore();

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
        debugStore.startTime = null;
    }

    function nowAsDayTime(): DayTime {
        const now = new Date();

        const milliseconds = now.getMilliseconds();

        return {
            hours: now.getHours(),
            minutes: now.getMinutes(),
            seconds: now.getSeconds(),
            fractional_part_in_ten_thousands: Math.floor((milliseconds / 1000) * 10000),
        };
    }

    function startClock() {
        if (mainStore.devMainHeatStartList != null) {
            const heat = mainStore.devMainHeatStartList;
            const heatId = heat.id;

            mainStore.sendGenericWSCommand({
                type: "DevStartRace",
                data: {
                    application: "DevTest",
                    version: "0.0.0",
                    generated: "2026-01-01T10:10:10",
                    time: nowAsDayTime(),
                    id: heatId,
                },
            } as MessageFromWebControlDevStartRace);
            debugStore.startTime = Date.now();
            debugStore.finishIndex = -1;
            debugStore.times = [];
        } else {
            console.error("No startlist... Sad");
        }
    }

    function sendWind() {
        if (mainStore.devMainHeatStartList != null && debugStore.startTime != null) {
            const heat = mainStore.devMainHeatStartList;
            const heatId = heat.id;

            mainStore.sendGenericWSCommand({
                type: "DevSendWind",
                data: {
                    application: "DevTest",
                    version: "0.0.0",
                    generated: "2026-01-01T10:10:10",
                    id: heatId,
                    wind: {
                        back_wind: Math.random() > 0.5,
                        whole_number_part: Math.round(Math.random() * 2),
                        fraction_part: Math.floor(Math.random() * 10),
                    },
                },
            } as MessageFromWebControlDevSendWind);
        } else {
            console.error("None started... Sad");
        }
    }

    function intermediateSignal() {
        if (mainStore.devMainHeatStartList != null && debugStore.startTime != null) {
            const heat = mainStore.devMainHeatStartList;
            const heatId = heat.id;

            const time = (Date.now() - debugStore.startTime) / 1000;

            mainStore.sendGenericWSCommand({
                type: "DevSendIntermediateSignal",
                data: {
                    application: "DevTest",
                    version: "0.0.0",
                    generated: "2026-01-01T10:10:10",
                    time: {
                        // this is not really used, we just send fake data here
                        hours: 10,
                        minutes: 10,
                        seconds: 10,
                        fractional_part_in_ten_thousands: null,
                    },
                    id: heatId,
                    intermediate_time_at: raceTimeFromNumber(time),
                },
            } as MessageFromWebControlDevSendIntermediateSignal);
        } else {
            console.error("None started... Sad");
        }
    }

    function endSignal() {
        if (mainStore.devMainHeatStartList != null && debugStore.startTime != null) {
            const heat = mainStore.devMainHeatStartList;
            const heatId = heat.id;

            const time = (Date.now() - debugStore.startTime) / 1000;

            mainStore.sendGenericWSCommand({
                type: "DevSendFinishSignal",
                data: {
                    application: "DevTest",
                    version: "0.0.0",
                    generated: "2026-01-01T10:10:10",
                    time: nowAsDayTime(),
                    id: heatId,
                    race_time: raceTimeFromNumber(time),
                },
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
            if (debugStore.finishIndex < mainStore.devMainHeatStartList.competitors.length - 1) {
                debugStore.finishIndex += 1;
            }

            const comp = mainStore.devMainHeatStartList.competitors[debugStore.finishIndex];
            return comp;
        }
    }

    function evaluateOneAthlete() {
        if (mainStore.devMainHeatStartList != null && debugStore.startTime != null) {
            const heat = mainStore.devMainHeatStartList;
            const heatId = heat.id;
            const last = getAthleteAtIndex();

            const time = (Date.now() - debugStore.startTime) / 1000;
            if (debugStore.times.length < heat.competitors.length) {
                debugStore.times.push(time);
            }
            let lastTime = debugStore.times[debugStore.times.length - 1];
            if (heat.competitors.length == 0) {
                lastTime = time;
            }

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
                        finish_time: nowAsDayTime(),
                        rank: 1, // this is not really used, we just send fake data here
                        runtime: raceTimeFromNumber(lastTime),
                        runtime_full_precision: raceTimeFromNumber(lastTime),
                    },
                },
            } as MessageFromWebControlDevSendEvaluated);

            const results = debugStore.times.map((time, index): HeatCompetitorResult => {
                const comp = heat.competitors[index];
                return {
                    competitor: comp,
                    difference_to_previous: { type: "Winner" }, // this is not really used, we just send fake data here
                    difference_to_winner: { type: "Winner" }, // this is not really used, we just send fake data here
                    distance: heat.distance_meters,
                    finish_time: nowAsDayTime(),
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
                    start_time: nowAsDayTime(),
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
