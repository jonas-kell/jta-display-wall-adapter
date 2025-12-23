<template>
    <v-tooltip :text="time" :disabled="time == ''">
        <template v-slot:activator="{ props }">
            <v-icon v-bind="props" icon="mdi-circle" :color="color"></v-icon>
        </template>
    </v-tooltip>
</template>

<script setup lang="ts">
    import { computed } from "vue";

    const props = defineProps<{ set: boolean; done: boolean; time: string }>();

    const color = computed(() => {
        return props.done ? "green" : props.set ? "yellow" : "red";
    });
</script>

<style scoped></style>
