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
            <v-btn @click="generatePDFcomponent" class="mr-2">Generate test</v-btn>
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
    import { ref, watch } from "vue";
    import useMainStore from "../stores/main";
    import PDFViewer from "./PDFViewer.vue";
    import { generatePDF } from "../functions/pdf";

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

    function generatePDFcomponent() {
        viewer.value?.setPDFtoRender(generatePDF(processedBackgroundImage.value));
    }

    // USE
    mainStore.sendUpsertPDBSettingCommand;
    mainStore.sendDeletePDFSettingCommand;
    mainStore.pdfConfigurationSettings;
</script>

<style scoped></style>
