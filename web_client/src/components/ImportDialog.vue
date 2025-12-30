<script setup lang="ts">
    import { ref } from "vue";
    import Papa from "papaparse";
    import { Person } from "../functions/importInterface";

    const dialog = ref(false);
    const csvText = ref("");
    const error = ref<string | null>(null);

    const emit = defineEmits<{
        (e: "finishedParsing", data: Person[]): void;
    }>();

    function openDialog() {
        dialog.value = true;
        csvText.value = "";
        error.value = null;
    }

    function confirmImport() {
        error.value = null;

        const result = Papa.parse<string[]>(csvText.value, {
            skipEmptyLines: true,
            delimiter: ";",
        });

        if (result.errors.length > 0) {
            error.value = result.errors[0].message;
            return;
        }

        try {
            const parsed: Person[] = result.data.map((row: string[], index: number) => {
                if (row.length < 3) {
                    throw new Error(`Row ${index + 1} has fewer than 3 columns`);
                }

                return {
                    firstName: row[0],
                    lastName: row[1],
                    birthDate: row[2],
                };
            });

            emit("finishedParsing", parsed);
            dialog.value = false;
        } catch (e) {
            error.value = e instanceof Error ? e.message : "Failed to parse CSV";
        }
    }
</script>

<template>
    <v-btn class="ml-5" @click="openDialog"> Import CSV </v-btn>

    <v-dialog v-model="dialog" max-width="600">
        <v-card>
            <v-card-title>Import CSV Data</v-card-title>

            <v-card-text>
                <p>
                    Paste CSV data below. Expected format:
                    <br />
                    <code>firstName;lastName;birthDate in yyyy-mm-dd</code>
                </p>

                <v-textarea v-model="csvText" label="CSV Data" rows="8" auto-grow />

                <v-alert v-if="error" type="error" class="mt-3" density="compact">
                    {{ error }}
                </v-alert>
            </v-card-text>

            <v-card-actions>
                <v-spacer />

                <v-btn variant="text" @click="dialog = false"> Cancel </v-btn>

                <v-btn color="primary" @click="confirmImport"> Confirm </v-btn>
            </v-card-actions>
        </v-card>
    </v-dialog>
</template>
