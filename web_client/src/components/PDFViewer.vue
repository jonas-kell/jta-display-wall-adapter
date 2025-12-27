<template>
    <div style="border: 1px solid black" :style="{ width: rendererWidth + 'px' }">
        <p v-if="pdfDataURI == null">Nothing Rendered</p>
        <VuePdfEmbed v-else :source="pdfDataURI" ref="pdf" v-on:rendered="handleFinishedRendering" :page="pageSelection" />
    </div>
</template>

<script setup lang="ts">
    import { ref } from "vue";
    import VuePdfEmbed from "vue-pdf-embed";

    const emit = defineEmits<{
        (e: "backgroundProcessed", value: { background: string; BGlandscape: boolean }): void;
    }>();

    const pdf = ref<InstanceType<typeof VuePdfEmbed>>();

    let pdfPrintBuffer = null as string | null;
    let nextStepPrint = false;
    let nextStepBG = false;
    function handleFinishedRendering() {
        if (nextStepPrint) {
            handleStepsForImmediatePrinting();

            // reset buffer
            setTimeout(() => {
                nextStepPrint = false;
                nextStepBG = false;
                pdfDataURI.value = pdfPrintBuffer;
            }, 500);
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
                emit("backgroundProcessed", {
                    background: imgData,
                    BGlandscape: bgLandscape,
                });

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

    function handleStepsForImmediatePrinting() {
        console.log("Printing");

        if (pdf.value) {
            pdf.value.print(600, 'Deselect "Headers and Footers to not see this"', true);
        } else {
            console.error("Ref not found");
        }
    }

    function processPDFasBackgroundToImageData(pdfData: string) {
        console.log("Processing Background pdf");
        nextStepPrint = false;
        nextStepBG = true;
        rendererWidth.value = BACKGROUND_GENERATION_RENDERER_WIDTH;
        pageSelection.value = 1;
        pdfDataURI.value = pdfData;
    }
    function setPDFtoRender(pdfData: string) {
        nextStepPrint = false;
        nextStepBG = false;

        pdfDataURI.value = pdfData;
    }
    function print(toPrint: string) {
        console.log("Print custom called");
        if (toPrint != pdfDataURI.value) {
            pdfPrintBuffer = pdfDataURI.value;
            nextStepPrint = true;
            pdfDataURI.value = toPrint;
        } else {
            handleStepsForImmediatePrinting();
        }
    }
    function printCurrentContent() {
        handleStepsForImmediatePrinting();
    }

    const DEFAULT_RENDERER_WIDTH = 400;
    const BACKGROUND_GENERATION_RENDERER_WIDTH = 1000;
    const rendererWidth = ref(DEFAULT_RENDERER_WIDTH);
    const pdfDataURI = ref(null as null | string);
    const pageSelection = ref(undefined as undefined | number | number[]);

    defineExpose({
        print,
        processPDFasBackgroundToImageData,
        setPDFtoRender,
        printCurrentContent,
    });
</script>

<style scoped></style>
