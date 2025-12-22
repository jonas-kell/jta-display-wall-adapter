<template>
    <h2>Home</h2>

    Welcome to the web-control of the <b>JTA Display Wall Adapter</b>

    <h3 class="mt-4">Query Heats</h3>

    <v-btn @click="mainStore.sendGetHeatsCommand">Get Heats</v-btn>
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
