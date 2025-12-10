<template>
    <h1>JTA Display Wall Adapter</h1>
    Wind Connected: {{ mainStore.windServerConnected }}, {{ mainStore.windTime }}
    <br />
    <br />

    Wind date: <input type="date" v-model="windDate" />
    <br />
    Wind from time: <input type="time" min="00:00" max="24:00" step="1" v-model="windFrom" />
    <br />
    How many seconds: <input type="number" min="5" max="20" v-model="duration" />
    <br />
    <br />
    <br />

    <button @click="sendWindRangeRequest" :disabled="!(windDate && windFrom && duration)">Request Wind for Range</button>
    <br />
    <br />
    {{ mainStore.requestedWindMeasurements }}
</template>

<script setup lang="ts">
    import { ref } from "vue";
    import useMainStore from "../stores/main";

    const today = new Date().toISOString().split("T")[0];

    const windDate = ref(today);
    const windFrom = ref(null);
    const duration = ref(10);

    function sendWindRangeRequest() {
        if (!windDate.value || !windFrom.value || duration.value == null) {
            console.error("Missing required parameters.");
            return;
        }

        const [hourStr, minStr, secStr] = String(windFrom.value).split(":");
        const start = new Date(
            Number(windDate.value.slice(0, 4)), // year
            Number(windDate.value.slice(5, 7)) - 1, // month index
            Number(windDate.value.slice(8, 10)), // day
            Number(hourStr),
            Number(minStr),
            Number(secStr),
            0
        );

        if (isNaN(start.getTime())) {
            console.error("Invalid start date or time.");
            return;
        }

        const end = new Date(start.getTime() + duration.value * 1000);

        const formatLocal = (d: Date) => {
            const pad = (n: number) => String(n).padStart(2, "0");
            return (
                `${d.getFullYear()}-` +
                `${pad(d.getMonth() + 1)}-` +
                `${pad(d.getDate())}T` +
                `${pad(d.getHours())}:` +
                `${pad(d.getMinutes())}:` +
                `${pad(d.getSeconds())}`
            );
        };

        const startStr = formatLocal(start);
        const endStr = formatLocal(end);

        mainStore.sendGetWindValuesCommand({
            from: startStr,
            to: endStr,
        });
    }

    const mainStore = useMainStore();
</script>

<style scoped></style>
