<template>
    <h2>Heats</h2>

    <v-btn @click="mainStore.sendGetHeatsCommand" class="mb-3">Get Heats</v-btn>
    <p v-for="heatEntry in mainStore.heatsMetaResult" class="mb-1">
        {{ heatEntry.name }}, Nr: {{ heatEntry.number }}, Time: {{ heatEntry.scheduled_start_time_string }}
        <v-btn
            @click="mainStore.sendSelectHeatCommand(heatEntry.id)"
            density="compact"
            :color="mainStore.selectedHeat && mainStore.selectedHeat.start_list.id == heatEntry.id ? 'green' : undefined"
            >Select</v-btn
        >
        <v-btn @click="mainStore.sendHeatToDisplayCommand(heatEntry.id)" density="compact" class="ml-1">Send To Display</v-btn>
        <v-btn
            @click="mainStore.sendSelectBibHeatCommand(heatEntry.id)"
            density="compact"
            class="ml-1"
            :color="
                mainStore.selectedHeatForBibMode && mainStore.selectedHeatForBibMode.heat_data.start_list.id == heatEntry.id
                    ? 'green'
                    : undefined
            "
            >Select for BIB</v-btn
        >
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
