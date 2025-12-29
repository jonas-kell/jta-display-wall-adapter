<template>
    <h2>PDF Test</h2>

    <div class="d-flex flex-row">
        <div>
            Background:
            <input type="file" @change="backgroundFileChange" accept="application/pdf" />
            <span v-if="processedBackgroundImage"
                >Available! ({{ processedBackgroundImageLandscape ? "Landscape" : "Portrait" }})</span
            >
            <br />
            <br />

            <h3>Bib</h3>
            <v-btn
                @click="
                    {
                        generatingCurrently = PDFSettingFor.Bib;
                        generatePDFcomponent();
                    }
                "
                class="mr-2"
                >Generate Bib</v-btn
            >
            <PDFSettingsTable class="mt-3" :settings="settingsBib" :for="PDFSettingFor.Bib"></PDFSettingsTable>
            <h3>Certificate</h3>
            <v-btn
                @click="
                    {
                        generatingCurrently = PDFSettingFor.Certificate;
                        generatePDFcomponent();
                    }
                "
                class="mr-2"
                >Generate Certificate</v-btn
            >
            <PDFSettingsTable class="mt-3" :settings="settingsCertificate" :for="PDFSettingFor.Certificate"></PDFSettingsTable>
        </div>

        <div class="d-flex flex-grow-1 justify-end mx-2">
            <PDFViewer
                ref="viewer"
                @background-processed="
                    (res) => {
                        processedBackgroundImage = res.background;
                        processedBackgroundImageLandscape = res.BGlandscape;
                    }
                "
            ></PDFViewer>
        </div>
    </div>
</template>

<script setup lang="ts">
    import { computed, ref, watch } from "vue";
    import useMainStore from "../stores/main";
    import PDFViewer from "./PDFViewer.vue";
    import PDFSettingsTable from "./PDFSettingsTable.vue";
    import { generatePDF } from "../functions/pdf";
    import { PDFSettingFor } from "../functions/interfaceShared";

    const viewer = ref<InstanceType<typeof PDFViewer>>();
    const mainStore = useMainStore();

    async function backgroundFileChange(event: Event) {
        const input = event.target as HTMLInputElement;
        if (!input.files || input.files.length === 0) return;

        const file: File = input.files[0];

        if (file.type !== "application/pdf") {
            console.error("Only PDF files are allowed");
            return;
        }

        let res = new Promise<string>((resolve, reject) => {
            const reader = new FileReader();

            reader.onload = () => {
                console.log("Read background pdf");
                resolve(reader.result as string);
            };

            reader.onerror = () => {
                console.error("File read failed");
                reject();
            };

            reader.readAsDataURL(file);
        });

        bgFile.value = await res;
    }

    let bgFile = ref(null as null | string);
    watch(bgFile, () => {
        if (bgFile.value) {
            viewer.value?.processPDFasBackgroundToImageData(bgFile.value);
            bgFile.value = null;
        }
    });

    const processedBackgroundImage = ref(null as null | string);
    const processedBackgroundImageLandscape = ref(false);

    const generatingCurrently = ref(null as null | PDFSettingFor);

    const settingsBib = computed(() => {
        return mainStore.pdfConfigurationSettings.filter((setting) => {
            return setting.for == PDFSettingFor.Bib;
        });
    });
    const settingsCertificate = computed(() => {
        return mainStore.pdfConfigurationSettings.filter((setting) => {
            return setting.for == PDFSettingFor.Certificate;
        });
    });

    function generatePDFcomponent() {
        if (generatingCurrently.value) {
            switch (generatingCurrently.value) {
                case PDFSettingFor.Bib:
                    viewer.value?.setPDFtoRender(generatePDF(true, processedBackgroundImage.value, settingsBib.value));
                    break;
                case PDFSettingFor.Certificate:
                    viewer.value?.setPDFtoRender(generatePDF(false, processedBackgroundImage.value, settingsCertificate.value));
                    break;
                default:
                    break;
            }
        }
    }

    watch(
        () => mainStore.pdfConfigurationSettings,
        () => {
            console.log("Re-rendering PDF");
            generatePDFcomponent();
        },
        {
            deep: true,
            immediate: true,
        }
    );
</script>

<style scoped></style>
