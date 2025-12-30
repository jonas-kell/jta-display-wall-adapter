<template>
    <h2>Manage Athletes</h2>

    <p>This will re-generate the .meetxml file in the configured folder:</p>
    <v-btn @click="mainStore.sendExportToFileCommand"> Export To File </v-btn>

    <h3 class="mt-4">
        Athletes for
        {{
            (mainStore.staticConfiguration?.mode ?? ApplicationMode.StreetLongRun) == ApplicationMode.StreetLongRun
                ? "Long-Run"
                : "Sprinterkönig"
        }}
        <template v-if="modeIsSPK">
            <div class="float-right d-flex align-center">
                <v-btn @click="generateSPKPDF" density="compact" class="mr-5"> Export Results </v-btn>
                <v-tooltip text="Show Sprinterkönig results" location="bottom center">
                    <template v-slot:activator="{ props }">
                        <v-switch
                            v-bind="props"
                            class="mr-5"
                            color="primary"
                            density="compact"
                            v-model="showSPKData"
                            hide-details
                        ></v-switch>
                    </template>
                </v-tooltip>
            </div>
        </template>
    </h3>

    <table>
        <thead>
            <tr>
                <th scope="col">Bib</th>
                <th scope="col">First Name</th>
                <th scope="col">Last Name</th>
                <th scope="col"></th>
                <th scope="col"></th>
                <!--from here street run data -->
                <template v-if="modeIsStreetRun">
                    <th>Rounds</th>
                    <th v-for="i in maxRoundsDisplay">Round {{ i }}</th>
                </template>
                <!--from here sprinterkönig data -->
                <template v-if="modeIsSPK">
                    <th>15-1</th>
                    <th>15-2</th>
                    <th>20-1</th>
                    <th>20-2</th>
                    <th>30-1</th>
                    <th>30-2</th>
                    <th>Guess</th>
                    <th>Time</th>
                    <th>Diff</th>
                    <th>Place</th>
                </template>
            </tr>
            <tr>
                <th scope="col"><input class="pl-2" type="number" v-model="bibRef" style="width: 100%" /></th>
                <th scope="col"><input class="pl-2" type="text" v-model="firstNameRef" style="width: 100%" /></th>
                <th scope="col"><input class="pl-2" type="text" v-model="lastNameRef" style="width: 100%" /></th>
                <th scope="col">
                    <v-tooltip text="Bib already used!" :disabled="bibAvailableForAdding">
                        <template v-slot:activator="{ props }">
                            <span v-bind="props">
                                <v-btn
                                    :icon="athleteBeingEdited ? 'mdi-content-save-outline' : 'mdi-plus'"
                                    density="compact"
                                    @click="addAthlete"
                                    :disabled="!canAddAthlete"
                                ></v-btn>
                            </span>
                        </template>
                    </v-tooltip>
                </th>
                <th></th>
                <!--from here street run data -->
                <template v-if="modeIsStreetRun">
                    <th style="width: 2cm">
                        <input class="pl-2" type="number" v-model="roundsRef" min="1" step="1" style="width: 100%" />
                    </th>
                    <th style="text-align: center" v-for="i in maxRoundsDisplay">
                        <v-btn
                            icon="mdi-delete"
                            density="compact"
                            @click="deleteEvaluation(i - 1)"
                            :disabled="!canDeleteEvaluation(i - 1)"
                            v-if="athleteBeingEdited"
                        ></v-btn>
                    </th>
                </template>
                <!--from here sprinterkönig data -->
                <template v-if="modeIsSPK">
                    <th></th>
                    <th></th>
                    <th></th>
                    <th></th>
                    <th></th>
                    <th></th>
                    <th style="width: 2cm">
                        <input class="pl-2" type="number" v-model="guessRef" min="0" step="0.01" style="width: 100%" />
                    </th>
                    <th></th>
                    <th></th>
                    <th></th>
                </template>
            </tr>
        </thead>
        <tbody>
            <tr v-for="athlete in athletesArraySorted">
                <td class="pl-2">{{ athlete.athlete.bib }}</td>
                <td class="pl-2">{{ athlete.athlete.first_name }}</td>
                <td class="pl-2">{{ athlete.athlete.last_name }}</td>
                <td style="text-align: center">
                    <v-btn
                        icon="mdi-pencil"
                        density="compact"
                        @click="editAthlete(athlete.athlete)"
                        :disabled="!canEditAthletes"
                    ></v-btn>
                </td>
                <td style="text-align: center">
                    <v-btn
                        icon="mdi-delete"
                        density="compact"
                        @click="deleteAthlete(athlete.athlete)"
                        :disabled="!canEditAthletes"
                    ></v-btn>
                </td>
                <!--from here street run data -->
                <template v-if="modeIsStreetRun">
                    <td style="width: 2cm; text-align: center">{{ athlete.athlete.street_run_rounds?.toFixed(0) ?? "" }}</td>
                    <td style="text-align: center" v-for="i in maxRoundsDisplay">
                        <StreetRunStateDot
                            :planned="roundPlanned(athlete.athlete.id, i - 1)"
                            :ran="roundRan(athlete.athlete.id, i - 1)"
                            :time="roundTime(athlete.athlete.id, i - 1)"
                        ></StreetRunStateDot>
                    </td>
                </template>
                <!--from here sprinterkönig data -->
                <template v-if="modeIsSPK">
                    <td style="text-align: center">
                        <SPKStateDot
                            :done="heatIsFinished(RunPossibilities.Run15_1, athlete)"
                            :set="heatIsDistributed(RunPossibilities.Run15_1, athlete)"
                            :time="formatForCircle(finishTimes[athlete.athlete.id][RunPossibilities.Run15_1])"
                        ></SPKStateDot>
                    </td>
                    <td style="text-align: center">
                        <SPKStateDot
                            :done="heatIsFinished(RunPossibilities.Run15_2, athlete)"
                            :set="heatIsDistributed(RunPossibilities.Run15_2, athlete)"
                            :time="formatForCircle(finishTimes[athlete.athlete.id][RunPossibilities.Run15_2])"
                        ></SPKStateDot>
                    </td>
                    <td style="text-align: center">
                        <SPKStateDot
                            :done="heatIsFinished(RunPossibilities.Run20_1, athlete)"
                            :set="heatIsDistributed(RunPossibilities.Run20_1, athlete)"
                            :time="formatForCircle(finishTimes[athlete.athlete.id][RunPossibilities.Run20_1])"
                        ></SPKStateDot>
                    </td>
                    <td style="text-align: center">
                        <SPKStateDot
                            :done="heatIsFinished(RunPossibilities.Run20_2, athlete)"
                            :set="heatIsDistributed(RunPossibilities.Run20_2, athlete)"
                            :time="formatForCircle(finishTimes[athlete.athlete.id][RunPossibilities.Run20_2])"
                        ></SPKStateDot>
                    </td>
                    <td style="text-align: center">
                        <SPKStateDot
                            :done="heatIsFinished(RunPossibilities.Run30_1, athlete)"
                            :set="heatIsDistributed(RunPossibilities.Run30_1, athlete)"
                            :time="formatForCircle(finishTimes[athlete.athlete.id][RunPossibilities.Run30_1])"
                        ></SPKStateDot>
                    </td>
                    <td style="text-align: center">
                        <SPKStateDot
                            :done="heatIsFinished(RunPossibilities.Run30_2, athlete)"
                            :set="heatIsDistributed(RunPossibilities.Run30_2, athlete)"
                            :time="formatForCircle(finishTimes[athlete.athlete.id][RunPossibilities.Run30_2])"
                        ></SPKStateDot>
                    </td>
                    <template v-if="showSPKData">
                        <td class="pl-1">{{ athlete.athlete.spk_guess?.toFixed(2) ?? "" }}</td>
                        <td class="pl-1">{{ (finalTimes[athlete.athlete.id] ?? ["", null])[1]?.toFixed(2) ?? "" }}</td>
                        <td class="pl-1">{{ (finalTimes[athlete.athlete.id] ?? ["", "", null])[2]?.toFixed(2) ?? "" }}</td>
                        <td class="pl-1">{{ places[athlete.athlete.id] ?? "" }}</td>
                    </template>
                    <template v-else>
                        <td class="pl-1">----</td>
                        <td class="pl-1">----</td>
                        <td class="pl-1">----</td>
                        <td class="pl-1">----</td>
                    </template>
                </template>
            </tr>
        </tbody>
    </table>

    <!--from here street run data -->
    <template v-if="modeIsStreetRun">
        <!-- space to do things -->
    </template>
    <!--from here sprinterkönig data -->
    <template v-if="modeIsSPK">
        <h3 class="mt-4">Heats</h3>
        <v-row class="pt-4 align-center">
            <v-select
                :items="runSelectionOptions"
                v-model="runSelection"
                density="compact"
                class="v-col-2"
                hide-details="auto"
            ></v-select>
            <v-combobox
                :items="selectableRunnersA"
                item-title="label"
                item-value="id"
                density="compact"
                v-model="selectedRunnerA"
                class="v-col-3"
                hide-details="auto"
                :auto-select-first="true"
            ></v-combobox>
            <v-combobox
                :items="selectableRunnersB"
                item-title="label"
                item-value="id"
                density="compact"
                v-model="selectedRunnerB"
                class="v-col-3"
                hide-details="auto"
                :auto-select-first="true"
            ></v-combobox>
            <v-btn class="v-col-1" :disabled="!heatCanBeAdded" @click="addHeatSPK"> ADD Heat </v-btn>
        </v-row>
        <table class="mt-2">
            <thead>
                <tr>
                    <th></th>
                    <th scope="col">Runner A</th>
                    <th scope="col">Runner B</th>
                    <th></th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="heat in heats">
                    <td>{{ heat[1].distance }}-{{ heat[1].heat_descriminator }}</td>
                    <td>{{ heat[0][0] }}</td>
                    <td>{{ heat[0][1] }}</td>
                    <td style="text-align: center">
                        <v-btn icon="mdi-delete" density="compact" @click="deleteHeat(heat[1])"></v-btn>
                    </td>
                </tr>
            </tbody>
        </table>
    </template>

    <v-btn class="mt-5" @click="debugDisplay" v-if="modeIsStreetRun"> Test Display </v-btn>
</template>

<script setup lang="ts">
    import { computed, watch } from "vue";
    import { ApplicationMode, Athlete, Gender, HeatAssignment } from "../functions/interfaceShared";
    import useMainStore from "../stores/main";
    import { ref } from "vue";
    import SPKStateDot from "./SPKStateDot.vue";
    import StreetRunStateDot from "./StreetRunStateDot.vue";
    import { AthleteWithMetadata } from "../functions/interfaceInbound";
    import { raceTimeStringRepr } from "../functions/representation";
    import jsPDF from "jspdf";
    import { uuid } from "../functions/uuid";
    import { RunPossibilities, sharedAthleteFunctionality } from "../functions/sharedAthleteTypes";

    const mainStore = useMainStore();
    const { athletesArray, finishTimes, evaluations } = sharedAthleteFunctionality();

    const idRef = ref(null as null | string);
    const bibRef = ref("");
    const lastNameRef = ref("");
    const firstNameRef = ref("");
    const guessRef = ref(""); // SPK
    const roundsRef = ref(""); // StreeRun

    const athletesByBib = computed(() => {
        return [...athletesArray.value].sort((a, b) => {
            return a.athlete.bib - b.athlete.bib;
        });
    });
    const athletesArraySorted = computed(() => {
        if (!finalTimesAvailable.value || !modeIsSPK.value || !showSPKData.value) {
            return athletesByBib.value;
        } else {
            return [...athletesByBib.value].sort((a, b) => {
                const aPlace: number | null = places.value[a.athlete.id] ?? null;
                const bPlace: number | null = places.value[b.athlete.id] ?? null;
                if (aPlace == null && bPlace != null) {
                    return 1;
                }
                if (aPlace != null && bPlace == null) {
                    return -1;
                }
                if (aPlace != null && bPlace != null) {
                    return aPlace - bPlace;
                }
                return a.athlete.bib - b.athlete.bib;
            });
        }
    });

    const canAddAthlete = computed(() => {
        return bibRef.value != "" && lastNameRef.value != "" && firstNameRef.value != "" && bibAvailableForAdding.value;
    });

    function editAthlete(ath: Athlete) {
        idRef.value = ath.id;

        bibRef.value = String(ath.bib);
        lastNameRef.value = ath.last_name;
        firstNameRef.value = ath.first_name;
        guessRef.value = String(ath.spk_guess ?? "");
        roundsRef.value = String(ath.street_run_rounds ?? "");
    }
    const canEditAthletes = computed(() => {
        return idRef.value == null;
    });
    const athleteBeingEdited = computed(() => {
        return idRef.value != null;
    });
    const bibAvailableForAdding = computed(() => {
        return athletesArray.value.every(
            (a) => a.athlete.bib != parseInt(bibRef.value) || (athleteBeingEdited.value && a.athlete.id == idRef.value)
        );
    });

    function deleteAthlete(ath: Athlete) {
        if (window.confirm(`Do you want to delete the athlete ${ath.first_name} ${ath.last_name}?`)) {
            mainStore.sendDeleteAthleteCommand(ath.id);
        }
    }

    // also does upsert
    function addAthlete() {
        const id = idRef.value ?? uuid();
        idRef.value = null;
        const updateBib = parseInt(bibRef.value);
        bibRef.value = "";
        const updateFirstName = firstNameRef.value;
        firstNameRef.value = "";
        const updateLastName = lastNameRef.value;
        lastNameRef.value = "";

        let spkGuess = null;
        if (guessRef.value) {
            spkGuess = parseFloat(guessRef.value);
        }
        guessRef.value = "";

        let streetRunRounds = null;
        if (roundsRef.value) {
            streetRunRounds = parseFloat(roundsRef.value);
        }
        roundsRef.value = "";

        const athlete: Athlete = {
            id: id,
            bib: updateBib,
            club: "placeholder", // TODO
            gender: Gender.Mixed, // TODO
            nation: "GER", // TODO
            first_name: updateFirstName,
            last_name: updateLastName,
            spk_guess: spkGuess,
            street_run_rounds: streetRunRounds,
        };

        mainStore.sendUpsertAthleteCommand(athlete);
    }

    // Street Run logic
    const modeIsStreetRun = computed(() => {
        return (mainStore.staticConfiguration?.mode ?? ApplicationMode.SprinterKing) == ApplicationMode.StreetLongRun;
    });
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
    // TODO do not compute ANY expensive mappings in a mode that does not require them (remove for others)
    const maxEvaluations = computed(() => {
        let max = 0;
        Object.values(evaluations.value).forEach((evals) => {
            if (evals.evaluations.length > max) {
                max = evals.evaluations.length;
            }
        });
        return max;
    });
    const maxRounds = computed(() => {
        let max = 0;
        Object.values(evaluations.value).forEach((aths) => {
            if (aths.athlete.street_run_rounds && aths.athlete.street_run_rounds > max) {
                max = aths.athlete.street_run_rounds;
            }
        });
        return max;
    });
    const maxRoundsDisplay = computed(() => {
        return Math.max(maxEvaluations.value, maxRounds.value);
    });
    function roundPlanned(athleteId: string, roundIndex: number): boolean {
        const dat = evaluations.value[athleteId];

        if (dat) {
            return (dat.athlete.street_run_rounds ?? 0) > roundIndex;
        }

        return false;
    }
    function roundRan(athleteId: string, roundIndex: number): boolean {
        const dat = evaluations.value[athleteId];

        if (dat) {
            return dat.evaluations.length > roundIndex;
        }

        return false;
    }
    function roundTime(athleteId: string, roundIndex: number): string {
        const dat = evaluations.value[athleteId];

        if (dat) {
            const run = dat.evaluations[roundIndex];
            if (run) {
                return raceTimeStringRepr(run.runtime_full_precision, false, false, 2);
            }
        }

        return "";
    }
    function deleteEvaluation(roundIndex: number) {
        const athleteId = idRef.value;
        if (!athleteId) {
            return;
        }

        const athlete = evaluations.value[athleteId];
        const evaluation = evaluations.value[athleteId].evaluations[roundIndex];

        if (
            window.confirm(
                `Do you want to delete the evaluation for ${athlete.athlete.first_name} ${
                    athlete.athlete.last_name
                }: ${raceTimeStringRepr(evaluation.runtime_full_precision, true, true, 4)}?`
            )
        ) {
            mainStore.sendDeleteCompetitorEvaluatedCommand(evaluation.finish_time);
        }
    }
    function canDeleteEvaluation(roundIndex: number): boolean {
        const athleteId = idRef.value;
        if (!athleteId) {
            return false;
        }

        const dat = evaluations.value[athleteId];

        if (dat) {
            if (dat.evaluations.length > roundIndex) {
                if (roundRan(athleteId, roundIndex)) {
                    return true;
                }
            }
        }

        return false;
    }

    // Sprinterkönig logic
    const modeIsSPK = computed(() => {
        return (mainStore.staticConfiguration?.mode ?? ApplicationMode.StreetLongRun) == ApplicationMode.SprinterKing;
    });
    const showSPKData = ref(true);
    function distanceFromPossibilities(d: RunPossibilities): number {
        switch (d) {
            case RunPossibilities.Run15_1:
                return 15;
            case RunPossibilities.Run15_2:
                return 15;
            case RunPossibilities.Run20_1:
                return 20;
            case RunPossibilities.Run20_2:
                return 20;
            case RunPossibilities.Run30_1:
                return 30;
            case RunPossibilities.Run30_2:
                return 30;
        }
    }
    function indexFromPossibilities(d: RunPossibilities): number {
        switch (d) {
            case RunPossibilities.Run15_1:
                return 1;
            case RunPossibilities.Run15_2:
                return 2;
            case RunPossibilities.Run20_1:
                return 1;
            case RunPossibilities.Run20_2:
                return 2;
            case RunPossibilities.Run30_1:
                return 1;
            case RunPossibilities.Run30_2:
                return 2;
        }
    }
    function formatForCircle(data: number | null): string {
        if (data != null) {
            return data.toFixed(2);
        } else {
            return "";
        }
    }
    const finalTimes = computed(() => {
        let res: { [key: string]: [number, number, number] | null } = {};
        athletesArray.value.forEach((athlete) => {
            const times = finishTimes.value[athlete.athlete.id];
            if (times) {
                const a = times[RunPossibilities.Run15_1];
                const b = times[RunPossibilities.Run15_2];
                const c = times[RunPossibilities.Run20_1];
                const d = times[RunPossibilities.Run20_2];
                const e = times[RunPossibilities.Run30_1];
                const f = times[RunPossibilities.Run30_2];
                if (a && b && c && d && e && f) {
                    const sum = a + b + c + d + e + f;
                    const guess = athlete.athlete.spk_guess;
                    if (guess) {
                        res[athlete.athlete.id] = [guess, sum, Math.abs(guess - sum)];
                        return;
                    }
                }
            }

            res[athlete.athlete.id] = null;
        });

        return res;
    });
    const finalTimesAvailable = computed(() => {
        return Object.values(finalTimes.value).some((e) => {
            return e != null;
        });
    });
    const places = computed(() => {
        let resIntermediate = [] as { id: string; diff: number }[];
        athletesArray.value.forEach((athlete) => {
            const final = finalTimes.value[athlete.athlete.id];

            if (final) {
                resIntermediate.push({
                    id: athlete.athlete.id,
                    diff: final[2],
                });
            }
        });
        resIntermediate.sort((a, b) => {
            return a.diff - b.diff;
        });

        let res: { [key: string]: number } = {};
        for (let index = 0; index < resIntermediate.length; index++) {
            const element = resIntermediate[index];
            res[element.id] = index + 1;
        }

        return res;
    });
    function heatIsDistributed(d: RunPossibilities, athlete: AthleteWithMetadata): boolean {
        return athlete.heat_assignments.some((ha) => {
            return ha.distance == distanceFromPossibilities(d) && ha.heat_descriminator == indexFromPossibilities(d);
        });
    }
    function heatIsFinished(d: RunPossibilities, athlete: AthleteWithMetadata): boolean {
        return finishTimes.value[athlete.athlete.id][d] != null;
    }
    const runSelection = ref(RunPossibilities.Run15_1);
    const runSelectionOptions = Object.values(RunPossibilities);
    watch(runSelection, () => {
        selectedRunnerA.value = null;
        selectedRunnerB.value = null;
    });
    const heats = computed(() => {
        let heats: HeatAssignment[] = [];

        athletesArray.value.forEach((a) => {
            a.heat_assignments.forEach((ha) => {
                if (
                    !heats.some((storedHeat) => {
                        return storedHeat.id == ha.id;
                    })
                ) {
                    heats.push(ha);
                }
            });
        });

        heats.sort((a, b) => {
            return b.id - a.id;
        });

        return heats.map((ha) => {
            const ida = ha.athlete_ids[1];
            const athleteA = athletesArray.value.find((a) => {
                return a.athlete.id == ida;
            });
            const idb = ha.athlete_ids[2];
            const athleteB = athletesArray.value.find((a) => {
                return a.athlete.id == idb;
            });
            let aName = "unknown";
            if (athleteA) {
                aName = athleteA.athlete.first_name + " " + athleteA.athlete.last_name;
            }
            let bName = "unknown";
            if (athleteB) {
                bName = athleteB.athlete.first_name + " " + athleteB.athlete.last_name;
            }

            return [[aName, bName], ha] as [[string, string], HeatAssignment];
        });
    });
    function deleteHeat(ha: HeatAssignment) {
        if (window.confirm(`Do you want to delete the heat ${ha.distance} ${ha.heat_descriminator}?`)) {
            mainStore.sendDeleteHeatAssignmentCommand(ha.id);
        }
    }
    // TODO make more general to support n runners
    const selectableRunnersA = computed(() => {
        const currentDistance = distanceFromPossibilities(runSelection.value);
        const currentIndex = indexFromPossibilities(runSelection.value);

        return athletesArray.value
            .map((a) => {
                return {
                    id: a.athlete.id,
                    label: a.athlete.first_name + " " + a.athlete.last_name,
                    heats: a.heat_assignments,
                };
            })
            .filter((a) => {
                return !a.heats.some((heat) => {
                    return heat.distance == currentDistance && heat.heat_descriminator == currentIndex;
                });
            })
            .map((a) => {
                return {
                    id: a.id,
                    label: a.label,
                };
            });
    });
    const selectedRunnerA = ref(null as null | { id: string; label: string });
    const selectableRunnersB = computed(() => {
        return selectableRunnersA.value.filter((a) => {
            return a.id != (selectedRunnerA.value?.id ?? "asdasdasd");
        });
    });
    const selectedRunnerB = ref(null as null | { id: string; label: string });
    const heatCanBeAdded = computed(() => {
        return selectedRunnerA.value != null && selectedRunnerB.value != null;
    });
    const EMPTY_UUID = "00000000-0000-0000-0000-000000000000";
    function addHeatSPK() {
        if (selectedRunnerA.value && selectedRunnerB.value) {
            const runnerAId = selectedRunnerA.value.id;
            selectedRunnerA.value = null;
            const runnerBId = selectedRunnerB.value.id;
            selectedRunnerB.value = null;

            mainStore.sendCreateHeatAssignmentCommand({
                athlete_ids: { 1: runnerAId, 2: runnerBId },
                id: -1, // is ignored on creation
                heat_id: EMPTY_UUID, // is ignored on creation
                distance: distanceFromPossibilities(runSelection.value),
                heat_descriminator: indexFromPossibilities(runSelection.value),
            });
        }
    }
    function generateSPKPDF() {
        // A4 page
        const PAGE_HEIGHT = 210;
        const PAGE_WIDTH = 297;
        const doc = new jsPDF({ orientation: "l", unit: "mm", format: [PAGE_WIDTH, PAGE_HEIGHT] });
        const TEXT_FONT = "times";
        const TEXT_SIZE = 15;

        // header
        doc.setFont(TEXT_FONT, "normal"); // also bold or italic
        doc.setFontSize(TEXT_SIZE);
        doc.text("Sprinterkönig Ergebnisse " + mainStore.staticConfiguration?.date, 10, 10);

        // table
        const placeKey = "Platz";
        const differenceKey = "Diff.";
        const guessedKey = "Getippt";
        const ranKey = "Gelaufen";
        const nameKey = "Name";
        const run1Key = "15m-1";
        const run2Key = "15m-2";
        const run3Key = "20m-1";
        const run4Key = "20m-2";
        const run5Key = "30m-1";
        const run6Key = "30m-2";
        const tableEntries = athletesArraySorted.value.map(
            (
                athlete
            ): {
                [key: string]: string;
            } => {
                let times = finalTimes.value[athlete.athlete.id] ?? [null, null, null];

                return {
                    [placeKey]: String(places.value[athlete.athlete.id] ?? " "),
                    [differenceKey]: String(times[2]?.toFixed(2) ?? " "),
                    [guessedKey]: String(times[0]?.toFixed(2) ?? " "),
                    [ranKey]: String(times[1]?.toFixed(2) ?? " "),
                    [nameKey]: athlete.athlete.first_name + " " + athlete.athlete.last_name,
                    [run1Key]: formatForCircle(finishTimes.value[athlete.athlete.id][RunPossibilities.Run15_1]),
                    [run2Key]: formatForCircle(finishTimes.value[athlete.athlete.id][RunPossibilities.Run15_2]),
                    [run3Key]: formatForCircle(finishTimes.value[athlete.athlete.id][RunPossibilities.Run20_1]),
                    [run4Key]: formatForCircle(finishTimes.value[athlete.athlete.id][RunPossibilities.Run20_2]),
                    [run5Key]: formatForCircle(finishTimes.value[athlete.athlete.id][RunPossibilities.Run30_1]),
                    [run6Key]: formatForCircle(finishTimes.value[athlete.athlete.id][RunPossibilities.Run30_2]),
                };
            }
        );
        doc.table(
            10,
            15,
            [...tableEntries],
            [placeKey, differenceKey, guessedKey, ranKey, nameKey, run1Key, run2Key, run3Key, run4Key, run5Key, run6Key],
            {
                autoSize: true,
                padding: 2,
            }
        );

        return doc.save("Sprinterkoenig_Ergebnisse.pdf");
    }
</script>

<style scoped></style>
