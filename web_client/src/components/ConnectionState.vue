<template>
    <div class="w-100 d-flex flex-row justify-space-between">
        <div class="d-flex" :class="props.collapsed ? 'flex-row' : 'flex-column'">
            <span class="py-1 px-2">
                Server:

                <v-tooltip
                    text="!!No changes are written to database, as database date does not match current date!!"
                    location="bottom center"
                    :disabled="rightDate"
                >
                    <template v-slot:activator="{ props }">
                        <v-icon
                            v-bind="props"
                            icon="mdi-circle"
                            :color="mainStore.connected ? (rightDate ? 'green' : 'yellow') : 'red'"
                        ></v-icon>
                    </template>
                </v-tooltip>
            </span>
            <span class="py-1 px-2">
                Display: <v-icon icon="mdi-circle" :color="mainStore.displayConnected ? 'green' : 'red'"></v-icon> (Mode:
                {{ mainStore.displayExternalPassthrough ? "External Passthrough" : "Default Client" }})
            </span>
            <span class="py-1 px-2">
                Wind: <v-icon icon="mdi-circle" :color="mainStore.windServerConnected ? 'green' : 'red'"></v-icon>
                {{ mainStore.windValue }}
                {{ mainStore.windServerConnected ? mainStore.windTime : "" }}
            </span>

            <v-btn
                @click="mainStore.sendSwitchModeCommand"
                :disabled="!mainStore.displayCanSwitchMode"
                density="comfortable"
                class="ma-2"
                v-if="!props.collapsed"
                variant="tonal"
            >
                Switch Display Mode
            </v-btn>
        </div>
        <div v-if="!props.collapsed" class="d-flex flex-column mx-5">
            <div>
                Client Display<span v-if="mainStore.displayExternalPassthrough"> (From External)</span>:
                <br />
                <template v-if="mainStore.displayConnected">
                    <img
                        width="360px"
                        style="max-height: 120px"
                        v-if="mainStore.currentClientFrame"
                        :src="mainStore.currentClientFrame"
                    />
                </template>
                <p v-else>No Connection</p>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
    import { computed } from "vue";
    import useMainStore from "../stores/main";
    import { TODAY } from "../functions/date";
    const mainStore = useMainStore();

    const props = defineProps<{ collapsed: boolean }>();

    const rightDate = computed(() => {
        return (mainStore.staticConfiguration?.date ?? "") == TODAY;
    });
</script>

<style scoped></style>
