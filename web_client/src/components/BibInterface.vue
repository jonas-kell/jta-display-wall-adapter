<template>
    <h2>Bib Interface <v-btn density="compact" to="/" v-if="!authStore.authenticated">Home</v-btn></h2>

    {{ mainStore.selectedHeatForBibMode }}
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import useAuthStore from "../stores/auth";

    const mainStore = useMainStore();
    const authStore = useAuthStore();

    authStore;
</script>

<style scoped></style>
