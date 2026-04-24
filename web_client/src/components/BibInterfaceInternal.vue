<template>
    <h2>Bib Results <v-btn density="compact" to="/" v-if="!authStore.authenticated">Home</v-btn></h2>

    <div v-if="mainStore.selectedHeatForBibMode != null">
        <h2 class="mb-5">Ausgewählt {{ mainStore.selectedHeatForBibMode.heat_data.start_list.name }}</h2>

        <table>
            <thead>
                <tr>
                    <th class="pl-2">Event</th>
                    <th class="pl-2">Time</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="row in rows">
                    <td class="pl-2">{{ row.event }}</td>
                    <td class="pl-2">{{ row.niceTime }}</td>
                </tr>
            </tbody>
        </table>
    </div>
    <h2 v-else color="red">Kein Lauf vorgelegt</h2>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import useAuthStore from "../stores/auth";
    import { computed } from "vue";
    import { numberFromRaceTime, raceTimeStringRepr } from "../functions/representation";

    const mainStore = useMainStore();
    const authStore = useAuthStore();

    type TableEntry = {
        event: string;
        time: number;
        niceTime: string;
    };
    const rows = computed<TableEntry[]>(() => {
        if (mainStore.selectedHeatForBibMode == null) {
            return [];
        }

        let res: TableEntry[] = [];

        let heat = mainStore.selectedHeatForBibMode;
        heat.bib_data_points.forEach((bdp) => {
            res.push({
                event: "Bib detected: " + bdp.bib,
                time: numberFromRaceTime(bdp.race_time),
                niceTime: raceTimeStringRepr(bdp.race_time, false, false, 1),
            });
        });

        res.sort((a, b) => {
            return a.time - b.time;
        });

        return res;
    });
</script>

<style scoped></style>
