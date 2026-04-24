<template>
    <h2>
        Bib Interface <v-btn density="compact" to="/" v-if="!authStore.authenticated">Home</v-btn>

        <v-tooltip text="Zeige Namen">
            <template v-slot:activator="{ props }">
                <v-switch v-bind="props" inset color="primary" v-model="showNames" hide-details class="float-right mr-5">
                </v-switch>
            </template>
        </v-tooltip>
    </h2>

    <div v-if="mainStore.selectedHeatForBibMode != null">
        <h2 class="mb-5">Ausgewählt {{ mainStore.selectedHeatForBibMode.heat_data.start_list.name }}</h2>

        <p v-for="comp in competitorsSorted" class="mb-3">
            <v-btn
                icon="mdi-repeat"
                @click="mainStore.recordBibEvent(comp.bib)"
                :disabled="!canRecordBibEvents || finishBibBlockedBecauseOfDirectOrEquivalentBlock.includes(comp.bib)"
                :color="finishBibBlockedBecauseOfDirectOrEquivalentBlock.includes(comp.bib) ? 'green' : undefined"
            ></v-btn>

            {{ getNumberOfRoundsForCompetitor(comp.bib) }} <span class="mr-3">Runden</span>

            <template v-for="eq in mainStore.selectedHeatForBibMode.equivalences">
                <b class="ml-2" v-if="eq.finish_bib == comp.bib">{{ eq.alternative_bib }}</b>
            </template>
            <b class="ml-2">{{ comp.bib }}</b>
            <span v-if="showNames" class="ml-4">{{ comp.last_name + " " + comp.first_name }}</span>
        </p>
    </div>
    <h2 v-else color="red">Kein Lauf vorgelegt</h2>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import useAuthStore from "../stores/auth";
    import { computed, ref, watch } from "vue";

    const mainStore = useMainStore();
    const authStore = useAuthStore();

    const SHOW_NAMES_STORAGE_KEY = "storing_name_key";

    const showNames = ref((localStorage.getItem(SHOW_NAMES_STORAGE_KEY) ?? "false") == "true");
    watch(showNames, () => {
        localStorage.setItem(SHOW_NAMES_STORAGE_KEY, String(showNames.value));
    });

    function getNumberOfRoundsForCompetitor(finishBib: number): number {
        if (mainStore.selectedHeatForBibMode == null) {
            return 0;
        } else {
            let count = 0;
            mainStore.selectedHeatForBibMode.bib_data_points.forEach((dp) => {
                if (dp.bib == finishBib) {
                    count++;
                } else {
                    if (
                        (mainStore.selectedHeatForBibMode?.equivalences ?? []).some(
                            (elem) => elem.finish_bib == finishBib && elem.alternative_bib == dp.bib,
                        )
                    ) {
                        count++;
                    }
                }
            });

            return count;
        }
    }

    const competitorsSorted = computed(() => {
        let arr = [...(mainStore.selectedHeatForBibMode?.heat_data.start_list.competitors ?? [])];

        arr.sort((a, b) => {
            return a.bib - b.bib;
        });
        // TODO more clever filter and ordering

        return arr;
    });

    const canRecordBibEvents = computed(() => {
        return mainStore.selectedHeatForBibMode != null && mainStore.selectedHeatForBibMode.heat_data.start != null;
    });

    const finishBibBlockedBecauseOfDirectOrEquivalentBlock = computed(() => {
        if (mainStore.selectedHeatForBibMode == null) {
            return [];
        } else {
            let blocked = [] as number[];
            mainStore.bibBlocks.forEach((blk) => {
                if ((mainStore.selectedHeatForBibMode?.heat_data.start_list.competitors ?? []).some((a) => a.bib == blk)) {
                    blocked.push(blk);
                } else {
                    (mainStore.selectedHeatForBibMode?.equivalences ?? []).forEach((eq) => {
                        if (eq.alternative_bib == blk) {
                            blocked.push(eq.finish_bib);
                        }
                    });
                }
            });

            return blocked;
        }
    });

    authStore;
</script>

<style scoped></style>
