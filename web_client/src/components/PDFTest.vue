<template>
    <h1>JTA Display Wall Adapter</h1>

    Background:
    <input type="file" @change="backgroundFileChange" accept="application/pdf" />
    <span v-if="processedBackgroundImage">Available! ({{ processedBackgroundImageLandscape ? "Landscape" : "Portrait" }})</span>

    <br />
    <br />
    <button @click="pdfDataURI = generatePDF(true)">Generate test</button>
    <button @click="print" :disabled="pdfDataURI == null">Print test</button>
    <br />
    <br />

    <div style="border: 1px solid black" :style="{ width: rendererWidth + 'px' }">
        <p v-if="pdfDataURI == null">Nothing Rendered</p>
        <VuePdfEmbed v-else :source="pdfDataURI" ref="pdf" v-on:rendered="handleFinishedRendering" :page="pageSelection" />
    </div>
</template>

<script setup lang="ts">
    import { ref, watch } from "vue";
    import useMainStore from "../stores/main";
    import VuePdfEmbed from "vue-pdf-embed";
    import { jsPDF } from "jspdf";

    const mainStore = useMainStore();

    const pdf = ref<InstanceType<typeof VuePdfEmbed>>();

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
            console.log("Processing Background pdf");
            nextStepPrint = false;
            nextStepBG = true;
            rendererWidth.value = BACKGROUND_GENERATION_RENDERER_WIDTH;
            pageSelection.value = 1;
            pdfDataURI.value = bgFile.value;
            bgFile.value = null;
        }
    });

    let pdfPrintBuffer = null as string | null;
    function print() {
        pdfPrintBuffer = pdfDataURI.value;
        const toPrint = generatePDF(false);

        nextStepPrint = true;
        pdfDataURI.value = toPrint;
    }

    let nextStepPrint = false;
    let nextStepBG = false;
    function handleFinishedRendering() {
        if (nextStepPrint) {
            if (pdf.value) {
                pdf.value.print(600, 'Deselect "Headers and Footers to not see this"', true);
            }

            // reset buffer
            nextStepPrint = false;
            nextStepBG = false;
            pdfDataURI.value = pdfPrintBuffer;
        } else {
            if (nextStepBG) {
                let bgLandscape = false;
                const canvas = pdf.value?.$el.querySelector("canvas") as HTMLCanvasElement;
                if (!canvas) {
                    throw new Error("PDF canvas not found");
                }

                const width = canvas.width;
                const height = canvas.height;
                if (width > height) {
                    bgLandscape = true;
                }

                console.log("Processed Background Image");
                const imgData = canvas.toDataURL("image/png");
                processedBackgroundImage.value = imgData;
                processedBackgroundImageLandscape.value = bgLandscape;

                // reset buffer
                nextStepBG = false;
                nextStepPrint = false;
                rendererWidth.value = DEFAULT_RENDERER_WIDTH;
                pageSelection.value = undefined;
                pdfDataURI.value = null;
            } else {
                // normal case
                console.log("(Re)rendered");
            }
        }
    }

    const DEFAULT_RENDERER_WIDTH = 500;
    const BACKGROUND_GENERATION_RENDERER_WIDTH = 1000;
    const rendererWidth = ref(DEFAULT_RENDERER_WIDTH);

    const pdfDataURI = ref(null as null | string);
    const pageSelection = ref(undefined as undefined | number | number[]);
    const processedBackgroundImage = ref(null as null | string);
    const processedBackgroundImageLandscape = ref(false);

    function generatePDF(withBackground: boolean) {
        // A4 page
        const PAGE_HEIGHT = 297;
        const PAGE_WIDTH = 210;
        const doc = new jsPDF({ orientation: "p", unit: "mm", format: [PAGE_WIDTH, PAGE_HEIGHT] });

        if (withBackground && processedBackgroundImage.value) {
            const pageWidth = doc.internal.pageSize.getWidth();
            const pageHeight = doc.internal.pageSize.getHeight();
            doc.addImage(processedBackgroundImage.value, "PNG", 0, 0, pageWidth, pageHeight);
        }

        const TEXT_FONT = "times";
        const TEXT_SIZE = 13;

        // header
        doc.setFont(TEXT_FONT, "normal"); // also bold or italic
        doc.setFontSize(TEXT_SIZE);

        doc.text("asdasd", 20, 20, {});

        return doc.output("dataurlstring");
    }

    mainStore;
</script>

<style scoped></style>
