<template>
    <h1>JTA Display Wall Adapter</h1>

    <button @click="mainStore.sendAdvertisementsCommand">Test</button>

    {{ mainStore.connected }}
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";

    const mainStore = useMainStore();
</script>

<style scoped></style>
