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
    <template v-if="mainStore.selectedHeatForBibMode">
        <h2>Heat Selected (Bib): {{ mainStore.selectedHeatForBibMode.heat_data.start_list.name }}</h2>
        <div class="pl-8">
            <ul>
                <li v-for="cmp in mainStore.selectedHeatForBibMode.heat_data.start_list.competitors">
                    <b>{{ cmp.bib }}</b>
                    <v-btn
                        class="ml-2"
                        icon="mdi-plus"
                        density="compact"
                        @click="addBibAlternative(cmp.bib, mainStore.selectedHeatForBibMode.heat_data.start_list.id)"
                    ></v-btn>
                    <tempalate v-for="eq in mainStore.selectedHeatForBibMode.equivalences">
                        <tempalate v-if="eq.finish_bib == cmp.bib">
                            <b class="ml-2">{{ eq.alternative_bib }}</b>
                            <v-btn
                                class="ml-2"
                                icon="mdi-delete"
                                density="compact"
                                @click="
                                    deleteBibAlternative(
                                        cmp.bib,
                                        eq.alternative_bib,
                                        mainStore.selectedHeatForBibMode.heat_data.start_list.id,
                                    )
                                "
                            ></v-btn>
                        </tempalate>
                    </tempalate>
                </li>
            </ul>
        </div>
    </template>
    <template v-if="mainStore.selectedHeat">
        <h2>Heat Selected: {{ mainStore.selectedHeat.meta.name }}</h2>
        <pre>{{ mainStore.selectedHeat }}</pre>
    </template>
    <p v-else>No heat selected</p>
</template>

<script setup lang="ts">
    import { Uuid } from "../generated/interface";
    import useMainStore from "../stores/main";
    const mainStore = useMainStore();

    function addBibAlternative(toBib: number, heatId: Uuid) {
        const newBib = prompt("Want to add an alternative to " + toBib + "?");

        if (newBib != null) {
            const parsed = parseInt(newBib);

            mainStore.createBibEquivalence({
                alternative_bib: parsed,
                finish_bib: toBib,
                heat_id: heatId,
            });
        }
    }

    function deleteBibAlternative(toBib: number, altBib: number, heatId: Uuid) {
        mainStore.deleteBibEquivalence({
            alternative_bib: altBib,
            finish_bib: toBib,
            heat_id: heatId,
        });
    }
</script>

<style scoped></style>
