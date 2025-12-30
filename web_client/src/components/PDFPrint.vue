<template>
    <h2>PDF Print</h2>

    <div class="d-flex justify-space-between">
        <div>
            <h3>Bib</h3>
            <v-btn @click="generateBib" class="mr-2">Generate</v-btn>
            <v-btn @click="print" class="mr-2">Print</v-btn>

            <h3 class="mt-4">Certificate</h3>
            <v-btn @click="generateCertificate" class="mr-2">Generate</v-btn>
            <v-btn @click="print" class="mr-2">Print</v-btn>
        </div>
        <PDFViewer ref="viewer"></PDFViewer>
    </div>
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import PDFViewer from "./PDFViewer.vue";
    import { generatePDF } from "../functions/pdf";
    import { ref, watch } from "vue";
    import { backgroundFileManagement } from "../functions/backgroundFiles";
    import { PDFSettingFor } from "../functions/interfaceShared";
    const mainStore = useMainStore();

    const { processedBackgroundImageBib, processedBackgroundImageCertificate } = backgroundFileManagement();

    const viewer = ref<InstanceType<typeof PDFViewer>>();

    const currentPDF = ref(null as string | null);

    watch([viewer, currentPDF], () => {
        if (currentPDF.value) {
            viewer.value?.setPDFtoRender(currentPDF.value);
        }
    });

    function generateBib() {
        currentPDF.value = generatePDF(
            true,
            processedBackgroundImageBib.value,
            mainStore.pdfConfigurationSettings.filter((set) => set.for == PDFSettingFor.Bib)
        );
    }
    function generateCertificate() {
        currentPDF.value = generatePDF(
            false,
            processedBackgroundImageCertificate.value,
            mainStore.pdfConfigurationSettings.filter((set) => set.for == PDFSettingFor.Certificate)
        );
    }

    function print() {
        viewer.value?.printCurrentContent();
        // or
        // viewer.value?.print(<a value to set>);
    }
</script>

<style scoped></style>
