<template>
    <h2>PDF Print</h2>

    <div class="d-flex justify-space-between">
        <div>
            <h3>Bib</h3>
            <v-btn @click="generateBib(false)" class="mr-2">Generate</v-btn>
            <v-btn @click="generateBib(true)" class="mr-2">Download</v-btn>
            <!--<v-btn @click="print" class="mr-2">Print</v-btn>-->

            <h3 class="mt-4">
                Certificate

                <div class="float-right">
                    <v-tooltip text="Hide non-finishers" location="bottom center">
                        <template v-slot:activator="{ props }">
                            <v-switch
                                v-bind="props"
                                class="mr-5"
                                color="primary"
                                density="compact"
                                v-model="hideNonFinishers"
                                hide-details
                            ></v-switch>
                        </template>
                    </v-tooltip>
                </div>
            </h3>
            <v-btn @click="generateCertificate(false)" class="mr-2">Generate</v-btn>
            <v-btn @click="generateCertificate(true)" class="mr-2">Download</v-btn>
            <!--<v-btn @click="print" class="mr-2">Print</v-btn>-->

            <p class="mt-5">
                click on headers to sort <v-btn @click="selectAll" class="mr-2" density="compact">Select all</v-btn> (sorted:
                {{ sortBy }}, {{ sortDir ? "desc" : "asc" }})
            </p>

            <table>
                <thead>
                    <tr>
                        <th scope="col"></th>
                        <th
                            scope="col"
                            @click="
                                () => {
                                    sortDir = !sortDir;
                                    sortBy = 'bib';
                                }
                            "
                        >
                            Bib
                        </th>
                        <th
                            scope="col"
                            @click="
                                {
                                    sortDir = !sortDir;
                                    sortBy = 'first';
                                }
                            "
                        >
                            Firstname
                        </th>
                        <th
                            scope="col"
                            @click="
                                {
                                    sortDir = !sortDir;
                                    sortBy = 'last';
                                }
                            "
                        >
                            Lastname
                        </th>
                        <th
                            scope="col"
                            @click="
                                {
                                    sortDir = !sortDir;
                                    sortBy = 'age';
                                }
                            "
                        >
                            Bdate
                        </th>
                        <th
                            scope="col"
                            @click="
                                {
                                    sortDir = !sortDir;
                                    sortBy = 'rounds';
                                }
                            "
                        >
                            # Rounds
                        </th>
                        <th
                            scope="col"
                            @click="
                                {
                                    sortDir = !sortDir;
                                    sortBy = 'time';
                                }
                            "
                        >
                            Fin. Time
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="athlete in athleteDataSorted">
                        <td class="unselectable">
                            <v-checkbox
                                hide-details="auto"
                                density="compact"
                                :value="athlete.id"
                                v-model="selectedOptionsAthletes"
                            ></v-checkbox>
                        </td>
                        <td scope="col" class="px-1">
                            {{ athlete.bib }}
                        </td>
                        <td scope="col" class="px-1">
                            {{ athlete.firstName }}
                        </td>
                        <td scope="col" class="px-1">
                            {{ athlete.lastName }}
                        </td>
                        <td scope="col" class="px-1">
                            {{ athlete.birthDate }}
                        </td>
                        <td scope="col" class="px-1">
                            {{ athlete.roundTimes.length }}
                        </td>
                        <td scope="col" class="px-1">
                            {{
                                athlete.roundTimes.length > 0
                                    ? raceTimeStringRepr(athlete.roundTimes[athlete.roundTimes.length - 1], true, true, 2)
                                    : ""
                            }}
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
        <PDFViewer ref="viewer"></PDFViewer>
    </div>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import PDFViewer from "./PDFViewer.vue";
    import { generatePDF } from "../functions/pdf";
    import { computed, ref, watch } from "vue";
    import { backgroundFileManagement } from "../functions/backgroundFiles";
    import { PDFSettingFor } from "../functions/interfaceShared";
    import { AthletePrintData, EvaluationsType, sharedAthleteFunctionality } from "../functions/sharedAthleteTypes";
    import { raceTimeStringRepr } from "../functions/representation";
    const mainStore = useMainStore();

    const { processedBackgroundImageBib, processedBackgroundImageCertificate } = backgroundFileManagement();
    const { evaluations } = sharedAthleteFunctionality();

    const viewer = ref<InstanceType<typeof PDFViewer>>();

    const currentPDF = ref(null as string | null);

    let sortBy = ref("bib" as "bib" | "first" | "last" | "age" | "rounds" | "time");
    let sortDir = ref(false);

    watch([viewer, currentPDF], () => {
        if (currentPDF.value) {
            viewer.value?.setPDFtoRender(currentPDF.value);
        }
    });

    function generateBib(download: boolean) {
        const res = generatePDF(
            download,
            true,
            processedBackgroundImageBib.value,
            mainStore.pdfConfigurationSettings.filter((set) => set.for == PDFSettingFor.Bib),
            athleteDataSortedFiltered.value
        );
        if (res) {
            currentPDF.value = res;
        }
    }
    function generateCertificate(download: boolean) {
        const res = generatePDF(
            download,
            false,
            processedBackgroundImageCertificate.value,
            mainStore.pdfConfigurationSettings.filter((set) => set.for == PDFSettingFor.Certificate),
            athleteDataSortedFiltered.value
        );
        if (res) {
            currentPDF.value = res;
        }
    }

    const selectedOptionsAthletes = ref([] as string[]);
    function selectAll() {
        if (selectedOptionsAthletes.value.length > 0) {
            selectedOptionsAthletes.value = [];
        } else {
            selectedOptionsAthletes.value = athleteData.value.map((a) => {
                return a.id;
            });
        }
    }

    const hideNonFinishers = ref(true);

    const evaluationsFiltered = computed((): EvaluationsType[] => {
        if (hideNonFinishers.value) {
            return Object.values(evaluations.value).filter((evaluation) => {
                return evaluation.evaluations.length > 0;
            });
        } else {
            return Object.values(evaluations.value);
        }
    });

    const athleteData = computed(() => {
        let res = [] as AthletePrintData[];
        evaluationsFiltered.value.forEach((evaluation) => {
            res.push({
                id: evaluation.athlete.id,
                bib: evaluation.athlete.bib,
                firstName: evaluation.athlete.first_name,
                lastName: evaluation.athlete.last_name,
                birthDate: evaluation.athlete.birth_date ?? "1800-01-01",
                roundTimes: evaluation.evaluations.map((e) => {
                    return e.runtime_full_precision;
                }),
                // roundTimes: [
                //     {
                //         hours: null,
                //         minutes: 12,
                //         seconds: 2,
                //         tenths: 1,
                //         hundrets: 5,
                //         ten_thousands: null,
                //         thousands: null,
                //     },
                //     {
                //         hours: null,
                //         minutes: 14,
                //         seconds: 3,
                //         tenths: 1,
                //         hundrets: 5,
                //         ten_thousands: null,
                //         thousands: null,
                //     },
                // ],
            });
        });

        return res;
    });

    const athleteDataSorted = computed(() => {
        let intermediate = [...athleteData.value];

        switch (sortBy.value) {
            case "age":
                intermediate = intermediate.sort((a, b) => {
                    return a.birthDate.localeCompare(b.birthDate);
                });
                break;
            case "bib":
                intermediate = intermediate.sort((a, b) => {
                    return a.bib - b.bib;
                });
                break;
            case "first":
                intermediate = intermediate.sort((a, b) => {
                    return a.firstName.localeCompare(b.firstName);
                });
                break;
            case "last":
                intermediate = intermediate.sort((a, b) => {
                    return a.lastName.localeCompare(b.lastName);
                });
                break;
            case "rounds":
                intermediate = intermediate.sort((a, b) => {
                    return a.roundTimes.length - b.roundTimes.length;
                });
                break;
            case "time":
                intermediate = intermediate.sort((a, b) => {
                    const aTime =
                        a.roundTimes.length > 0 ? raceTimeStringRepr(a.roundTimes[a.roundTimes.length - 1], true, true, 2) : null;
                    const bTime =
                        b.roundTimes.length > 0 ? raceTimeStringRepr(b.roundTimes[b.roundTimes.length - 1], true, true, 2) : null;

                    if (aTime) {
                        if (bTime) {
                            return aTime.localeCompare(bTime);
                        } else {
                            return 1;
                        }
                    } else {
                        if (bTime) {
                            return -1;
                        } else {
                            return 0;
                        }
                    }
                });
                break;

            default:
                break;
        }

        if (sortDir.value) {
            intermediate.reverse();
        }

        return intermediate;
    });

    const athleteDataSortedFiltered = computed(() => {
        return athleteDataSorted.value.filter((a) => {
            return selectedOptionsAthletes.value.includes(a.id);
        });
    });

    // prints double pages. No idea why TODO fix
    function print() {
        // viewer.value?.printCurrentContent();
        // or
        // viewer.value?.print(generate...());
    }
    print;
</script>

<style scoped></style>
