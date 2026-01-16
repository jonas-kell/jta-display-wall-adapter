<template>
    <h2>Home</h2>

    Welcome to the Web-Control of the <b>JTA Display Wall Adapter</b>

    <template v-if="!authStore.authenticated">
        <h3 class="mt-3">Applications</h3>

        <v-btn density="compact" to="/bib_interface">Bib Interface</v-btn>

        <h3 class="mt-3">Admin Mode</h3>
        <v-text-field v-model="authStore.password" label="Admin Password"></v-text-field>
    </template>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import useAuthStore from "../stores/auth";

    const mainStore = useMainStore();
    const authStore = useAuthStore();

    mainStore;
</script>

<style scoped></style>
