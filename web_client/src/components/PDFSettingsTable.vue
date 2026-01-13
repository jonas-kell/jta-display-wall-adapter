<template>
    <table>
        <thead>
            <tr>
                <th scope="col">X</th>
                <th scope="col">Y</th>
                <th scope="col">Size</th>
                <th scope="col">Bold</th>
                <th scope="col">Italic</th>
                <th scope="col">Centered</th>
                <th scope="col">Type</th>
                <th scope="col">Content</th>
                <th scope="col">Extra</th>
                <th scope="col"></th>
                <th scope="col"></th>
            </tr>
            <tr>
                <th scope="col"><input class="pl-2" type="number" v-model="xRef" style="width: 100%" /></th>
                <th scope="col"><input class="pl-2" type="number" v-model="yRef" style="width: 100%" /></th>
                <th scope="col"><input class="pl-2" type="number" v-model="sizeRef" style="width: 100%" /></th>
                <th scope="col"><input class="pl-2" type="checkbox" v-model="boldRef" style="width: 100%" /></th>
                <th scope="col"><input class="pl-2" type="checkbox" v-model="italicRef" style="width: 100%" /></th>
                <th scope="col"><input class="pl-2" type="checkbox" v-model="centeredRef" style="width: 100%" /></th>
                <th scope="col">
                    <v-select :items="types" v-model="typeRef" width="100%" density="compact" :hide-details="true"></v-select>
                </th>
                <th scope="col">
                    <input class="pl-2" type="text" v-model="contentRef" style="width: 100%" v-if="typeRef == 'Text'" />
                    <v-select
                        :items="Object.values(PDFConfigurationContentReferenceReference) as string[] ?? []"
                        v-model="contentRef"
                        width="100%"
                        density="compact"
                        :hide-details="true"
                        v-else
                    ></v-select>
                </th>
                <th scope="col">
                    <input class="pl-2" type="text" v-model="content2Ref" style="width: 100%" v-if="typeRef == 'Reference'" />
                </th>
                <th></th>
                <th scope="col">
                    <v-btn
                        :icon="settingBeingEdited ? 'mdi-content-save-outline' : 'mdi-plus'"
                        density="compact"
                        @click="addSetting"
                        :disabled="!canAddSetting"
                    ></v-btn>
                </th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="setting in settings">
                <td class="pl-2">{{ setting.pos_x }}</td>
                <td class="pl-2">{{ setting.pos_y }}</td>
                <td class="pl-2">{{ setting.size }}</td>
                <td class="pl-2">{{ setting.bold }}</td>
                <td class="pl-2">{{ setting.italic }}</td>
                <td class="pl-2">{{ setting.centered }}</td>
                <td class="pl-2">{{ setting.content.type == "PDFConfigurationContentText" ? "Text" : "Reference" }}</td>
                <td class="pl-2" v-if="setting.content.type == 'PDFConfigurationContentText'">
                    {{ setting.content.data.text }}
                </td>
                <td class="pl-2" v-if="setting.content.type == 'PDFConfigurationContentReference'">
                    {{ setting.content.data.reference }}
                </td>
                <td class="pl-2">
                    <template v-if="setting.content.type == 'PDFConfigurationContentReference'">
                        {{ setting.content.data.reference_content ?? "" }}
                    </template>
                </td>
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
            </tr>
        </tbody>
    </table>
</template>

<script setup lang="ts">
    import { PDFConfigurationContent, PDFConfigurationSetting, PDFSettingFor } from "../generated/interface";
    import { uuid } from "../functions/uuid";
    import useMainStore from "../stores/main";
    import { computed, ref } from "vue";
    import { PDFConfigurationContentReferenceReference } from "../functions/pdf";

    type FieldType = "Text" | "Reference";
    const types = ["Text", "Reference"] as FieldType[];

    const props = defineProps<{ settings: PDFConfigurationSetting[]; setting_for: PDFSettingFor }>();

    const mainStore = useMainStore();

    const idRef = ref(null as null | string);
    const xRef = ref("");
    const yRef = ref("");
    const boldRef = ref(false);
    const italicRef = ref(false);
    const centeredRef = ref(false);
    const sizeRef = ref("");
    const typeRef = ref("Text" as FieldType);
    const contentRef = ref("" as string | PDFConfigurationContentReferenceReference);
    const content2Ref = ref("");

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

        boldRef.value = set.bold;
        italicRef.value = set.italic;
        centeredRef.value = set.centered;
        sizeRef.value = String(set.size);
        typeRef.value = set.content.type == "PDFConfigurationContentText" ? "Text" : "Reference"; // TODO more dynamic
        contentRef.value = String(
            set.content.type == "PDFConfigurationContentText" ? set.content.data.text : set.content.data.reference
        );
        content2Ref.value = String(
            (set.content.type == "PDFConfigurationContentText" ? null : set.content.data.reference_content) ?? ""
        );
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
        const updateContent2 = content2Ref.value;
        content2Ref.value = "";
        const updateSize = parseInt(sizeRef.value);
        sizeRef.value = "";

        let content = null as null | PDFConfigurationContent;
        if (typeRef.value == "Text") {
            content = {
                type: "PDFConfigurationContentText",
                data: {
                    text: updateContent,
                },
            };
        }
        if (typeRef.value == "Reference") {
            content = {
                type: "PDFConfigurationContentReference",
                data: {
                    reference: updateContent,
                    reference_content: updateContent2 == null || updateContent2 == "" ? null : updateContent2,
                },
            };
        }

        if (content) {
            const setting: PDFConfigurationSetting = {
                id: id,
                pos_x: updateX,
                pos_y: updateY,
                bold: boldRef.value,
                italic: italicRef.value,
                size: updateSize,
                centered: centeredRef.value,
                setting_for: props.setting_for,
                content,
            };

            mainStore.sendUpsertPDFSettingCommand(setting);
        }
    }
</script>

<style scoped></style>
