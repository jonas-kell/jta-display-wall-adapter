<template>
    <h2>Bib Results <v-btn density="compact" to="/" v-if="!authStore.authenticated">Home</v-btn></h2>

    <div v-if="mainStore.selectedHeatForBibMode != null">
        <h2 class="mb-5">Ausgewählt {{ mainStore.selectedHeatForBibMode.heat_data.start_list.name }}</h2>

        <table>
            <thead>
                <tr>
                    <th class="pl-2">Event</th>
                    <th class="pl-2">Time</th>
                    <th class="pl-2">Request</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="row in rows">
                    <td class="pl-2" :style="{ backgroundColor: row.closest ? 'darkgreen' : undefined }">{{ row.event }}</td>
                    <td class="pl-2" :style="{ backgroundColor: row.closest ? 'darkgreen' : undefined }">{{ row.niceTime }}</td>
                    <td class="pl-2" :style="{ backgroundColor: row.closest ? 'darkgreen' : undefined }">{{ jumpTo }}</td>
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
        closest: boolean;
    };
    const rows = computed<TableEntry[]>(() => {
        if (mainStore.selectedHeatForBibMode == null) {
            return [];
        }

        let res: TableEntry[] = [];

        let closestTime = 99999999;
        let closestTimeDifference = 99999999;
        let targetTime = 999999;
        if (mainStore.bibJumpTo != null) {
            targetTime = numberFromRaceTime(mainStore.bibJumpTo);
        }

        let heat = mainStore.selectedHeatForBibMode;
        heat.bib_data_points.forEach((bdp) => {
            const timeAsNumber = numberFromRaceTime(bdp.race_time);
            const newDiff = Math.abs(timeAsNumber - targetTime);
            if (newDiff < closestTimeDifference) {
                closestTimeDifference = newDiff;
                closestTime = timeAsNumber;
            }
            res.push({
                event: "Bib detected: " + bdp.bib,
                time: timeAsNumber,
                niceTime: raceTimeStringRepr(bdp.race_time, false, false, 1),
                closest: false,
            });
        });

        res.sort((a, b) => {
            return a.time - b.time;
        });

        if (mainStore.bibJumpTo != null) {
            res.forEach((a) => {
                if (a.time == closestTime) {
                    a.closest = true;
                }
            });
        }

        return res;
    });

    const jumpTo = computed(() => {
        if (mainStore.bibJumpTo == null) {
            return "";
        } else {
            return raceTimeStringRepr(mainStore.bibJumpTo, false, false, 1);
        }
    });
</script>

<style scoped></style>
