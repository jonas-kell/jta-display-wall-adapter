<template>
    <h2>PDF Print</h2>

    <PDFViewer ref="viewer"></PDFViewer>
    <v-btn @click="print" class="mr-2">Print</v-btn>

    TODO
</template>

<script setup lang="ts">
    import useMainStore from "../stores/main";
    import PDFViewer from "./PDFViewer.vue";
    import { generatePDF } from "../functions/pdf";
    import { ref, watch } from "vue";

    const viewer = ref<InstanceType<typeof PDFViewer>>();

    const currentPDF = ref(generatePDF(null));

    watch([viewer, currentPDF], () => {
        // viewer.value?.setPDFtoRender(currentPDF.value);
    });

    function print() {
        // viewer.value?.printCurrentContent();
        // or
        viewer.value?.print(currentPDF.value);
    }

    const mainStore = useMainStore();

    // USE
    mainStore.athletesData;
    mainStore.pdfConfigurationSettings;
</script>

<style scoped></style>
