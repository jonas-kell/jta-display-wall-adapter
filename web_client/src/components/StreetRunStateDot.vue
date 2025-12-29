<template>
    <v-tooltip :text="time" :disabled="time == ''">
        <template v-slot:activator="{ props }">
            <v-icon v-bind="props" icon="mdi-circle" :color="color" v-if="!hide"></v-icon>
        </template>
    </v-tooltip>
</template>

<script setup lang="ts">
    import { computed } from "vue";

    const props = defineProps<{ planned: boolean; ran: boolean; time: string }>();

    const color = computed(() => {
        if (props.planned) {
            if (props.ran) {
                return "green";
            } else {
                return "red";
            }
        } else {
            if (props.ran) {
                return "yellow";
            } else {
                return ""; // HIDDEN
            }
        }
    });

    const hide = computed(() => {
        return !props.planned && !props.ran;
    });
</script>

<style scoped></style>
