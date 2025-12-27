<template>
    <h2>Heats</h2>

    <v-btn @click="mainStore.sendGetHeatsCommand" class="mb-3">Get Heats</v-btn>
    <p v-for="heatEntry in mainStore.heatsMetaResult">
        {{ heatEntry.name }}, Nr: {{ heatEntry.number }}, Time: {{ heatEntry.scheduled_start_time_string }}
        <v-btn @click="mainStore.sendSelectHeatCommand(heatEntry.id)" density="compact">Select</v-btn>
    </p>
    <p v-if="mainStore.heatsMetaResult.length == 0">No heats loaded/available</p>
    <br />
    <template v-if="mainStore.selectedHeat">
        Heat Selected: {{ mainStore.selectedHeat.meta.name }} <br />
        <pre>{{ mainStore.selectedHeat }}</pre>
    </template>
    <p v-else>No heat selected</p>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    const mainStore = useMainStore();
</script>

<style scoped></style>
