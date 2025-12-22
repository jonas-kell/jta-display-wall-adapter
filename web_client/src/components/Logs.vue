<template>
    <v-container class="d-flex justify-center">
        <v-btn @click="mainStore.sendGetLogsCommand(10000)" variant="tonal">Get Logs</v-btn>
    </v-container>
    <v-container>
        <p v-for="logEntry in mainStore.logEntries" class="mb-2">
            Key: {{ logEntry.name_key }}
            <br />
            Time: {{ logEntry.stored_at }}
            <br />
            Data: {{ JSON.parse(logEntry.data) }}
        </p>
        <p v-if="mainStore.logEntries.length == 0">No logs loaded/available</p>
    </v-container>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    const mainStore = useMainStore();
</script>

<style scoped></style>
