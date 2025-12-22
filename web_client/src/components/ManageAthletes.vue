<template>
    <h2>Manage Athletes</h2>

    <p>This will re-generate the .meetxml file in the configured folder:</p>
    <v-btn @click="mainStore.sendExportToFileCommand"> Export To File </v-btn>

    <h3 class="mt-4">
        Athletes for
        {{
            (mainStore.staticConfiguration?.mode ?? ApplicationMode.StreetLongRun) == ApplicationMode.StreetLongRun
                ? "Long-Run"
                : "Sprinterkönig"
        }}
    </h3>
</template>

<script setup lang="ts">
    import { ApplicationMode } from "../functions/interfaceShared";
    import useMainStore from "../stores/main";
    const mainStore = useMainStore();
</script>

<style scoped></style>
