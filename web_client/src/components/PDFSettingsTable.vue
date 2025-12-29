<template>
    <table>
        <thead>
            <tr>
                <th scope="col">X</th>
                <th scope="col">Y</th>
                <th scope="col">Type</th>
                <th scope="col"></th>
                <th scope="col"></th>
                <th scope="col">Content</th>
            </tr>
            <tr>
                <th scope="col"><input class="pl-2" type="number" v-model="xRef" style="width: 100%" /></th>
                <th scope="col"><input class="pl-2" type="number" v-model="yRef" style="width: 100%" /></th>
                <th scope="col">type</th>
                <th></th>
                <th scope="col">
                    <v-btn
                        :icon="settingBeingEdited ? 'mdi-content-save-outline' : 'mdi-plus'"
                        density="compact"
                        @click="addSetting"
                        :disabled="!canAddSetting"
                    ></v-btn>
                </th>
                <th scope="col"><input class="pl-2" type="text" v-model="contentRef" style="width: 100%" /></th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="setting in settings">
                <td class="pl-2">{{ setting.pos_x }}</td>
                <td class="pl-2">{{ setting.pos_y }}</td>
                <td class="pl-2">{{ setting.content.type }}</td>
                <td style="text-align: center">
                    <v-btn icon="mdi-pencil" density="compact" @click="editSetting(setting)" :disabled="!canEditSettings"></v-btn>
                </td>
                <td style="text-align: center">
                    <v-btn
                        icon="mdi-delete"
                        density="compact"
                        @click="deleteSetting(setting)"
                        :disabled="!canEditSettings"
                    ></v-btn>
                </td>
                <td class="pl-2" v-if="setting.content.type == 'PDFConfigurationContentText'">{{ setting.content.text }}</td>
                <td class="pl-2" v-if="setting.content.type == 'PDFConfigurationContentReference'">
                    {{ setting.content.reference }}
                </td>
            </tr>
        </tbody>
    </table>
</template>

<script setup lang="ts">
    import { PDFConfigurationSetting, PDFSettingFor } from "../functions/interfaceShared";
    import { uuid } from "../functions/uuid";
    import useMainStore from "../stores/main";
    import { computed, ref } from "vue";

    const props = defineProps<{ settings: PDFConfigurationSetting[]; for: PDFSettingFor }>();

    const mainStore = useMainStore();

    const idRef = ref(null as null | string);
    const xRef = ref("");
    const yRef = ref("");
    const contentRef = ref("");

    const canAddSetting = computed(() => {
        return xRef.value != "" && yRef.value != "";
    });
    const canEditSettings = computed(() => {
        return idRef.value == null;
    });
    const settingBeingEdited = computed(() => {
        return idRef.value != null;
    });

    function deleteSetting(set: PDFConfigurationSetting) {
        if (window.confirm(`Do you want to delete the setting at ${set.pos_x}-${set.pos_y}?`)) {
            mainStore.sendDeletePDFSettingCommand(set.id);
        }
    }

    function editSetting(set: PDFConfigurationSetting) {
        idRef.value = set.id;

        xRef.value = String(set.pos_x);
        yRef.value = String(set.pos_y);
    }

    // also does upsert
    function addSetting() {
        const id = idRef.value ?? uuid();
        idRef.value = null;
        const updateX = parseInt(xRef.value);
        xRef.value = "";
        const updateY = parseInt(yRef.value);
        yRef.value = "";
        const updateContent = contentRef.value;
        contentRef.value = "";

        const setting: PDFConfigurationSetting = {
            id: id,
            pos_x: updateX,
            pos_y: updateY,
            for: props.for,
            content: {
                type: "PDFConfigurationContentText",
                text: updateContent,
            },
        };

        mainStore.sendUpsertPDFSettingCommand(setting);
    }
</script>

<style scoped></style>
