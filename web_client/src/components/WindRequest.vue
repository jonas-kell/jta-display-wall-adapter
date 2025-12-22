<template>
    <v-text-field density="compact" type="date" v-model="windDate" label="Wind date" />
    <v-text-field density="compact" type="time" v-model="duration" min="00:00" max="24:00" step="1" label="Wind from time" />
    <v-text-field density="compact" type="number" min="5" max="20" v-model="duration" label="How many seconds" />

    <br />

    <v-btn @click="sendWindRangeRequest" :disabled="!(windDate && windFrom && duration)" density="comfortable"
        >Request Wind for Range</v-btn
    >
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
