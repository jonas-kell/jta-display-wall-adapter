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

    <table>
        <thead>
            <tr>
                <th scope="col">Bib</th>
                <th scope="col">First Name</th>
                <th scope="col">Last Name</th>
                <th scope="col"></th>
                <th scope="col"></th>
            </tr>
            <tr>
                <th scope="col"><input type="number" v-model="bibRef" style="width: 100%" /></th>
                <th scope="col"><input type="text" v-model="firstNameRef" style="width: 100%" /></th>
                <th scope="col"><input type="text" v-model="lastNameRef" style="width: 100%" /></th>
                <th scope="col">
                    <v-btn
                        :icon="athleteBeingEdited ? 'mdi-content-save-outline' : 'mdi-plus'"
                        density="compact"
                        @click="addAthlete"
                        :disabled="!canAddAthlete"
                    ></v-btn>
                </th>
                <th></th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="athlete in athletesByBib">
                <td>{{ athlete.athlete.bib }}</td>
                <td>{{ athlete.athlete.first_name }}</td>
                <td>{{ athlete.athlete.last_name }}</td>
                <td style="text-align: center">
                    <v-btn
                        icon="mdi-pencil"
                        density="compact"
                        @click="editAthlete(athlete.athlete)"
                        :disabled="!canEditAthletes"
                    ></v-btn>
                </td>
                <td style="text-align: center">
                    <v-btn
                        icon="mdi-delete"
                        density="compact"
                        @click="deleteAthlete(athlete.athlete)"
                        :disabled="!canEditAthletes"
                    ></v-btn>
                </td>
            </tr>
        </tbody>
    </table>
</template>

<script setup lang="ts">
    import { computed } from "vue";
    import { ApplicationMode, Athlete, Gender } from "../functions/interfaceShared";
    import useMainStore from "../stores/main";
    import { ref } from "vue";
    import { v4 as uuid } from "uuid";

    const mainStore = useMainStore();

    const idRef = ref(null as null | string);
    const bibRef = ref("");
    const lastNameRef = ref("");
    const firstNameRef = ref("");

    const athletesByBib = computed(() => {
        return [...mainStore.athletesData].sort((a, b) => {
            return a.athlete.bib - b.athlete.bib;
        });
    });

    const canAddAthlete = computed(() => {
        return bibRef.value != "" && lastNameRef.value != "" && firstNameRef.value != "";
    });

    function editAthlete(ath: Athlete) {
        idRef.value = ath.id;

        bibRef.value = String(ath.bib);
        lastNameRef.value = ath.last_name;
        firstNameRef.value = ath.first_name;
    }
    const canEditAthletes = computed(() => {
        return idRef.value == null;
    });
    const athleteBeingEdited = computed(() => {
        return idRef.value != null;
    });

    function deleteAthlete(ath: Athlete) {
        if (window.confirm(`Do you want to delete the athlete ${ath.first_name} ${ath.last_name}?`)) {
            mainStore.sendDeleteAthleteCommand(ath.id);
        }
    }

    // als odoes upsert
    function addAthlete() {
        const id = idRef.value ?? uuid();
        idRef.value = null;
        const updateBib = parseInt(bibRef.value);
        bibRef.value = "";
        const updateFirstName = firstNameRef.value;
        firstNameRef.value = "";
        const updateLastName = lastNameRef.value;
        lastNameRef.value = "";

        const athlete: Athlete = {
            id: id,
            bib: updateBib,
            club: "placeholder", // TODO
            gender: Gender.Mixed, // TODO
            nation: "GER", // TODO
            first_name: updateFirstName,
            last_name: updateLastName,
        };

        mainStore.sendUpsertAthleteCommand(athlete);
    }
</script>

<style scoped></style>
