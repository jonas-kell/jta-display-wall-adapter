import { jsPDF } from "jspdf";

export function generatePDF(bgImage: string | null) {
    // A4 page
    const PAGE_HEIGHT = 297;
    const PAGE_WIDTH = 210;
    const doc = new jsPDF({ orientation: "p", unit: "mm", format: [PAGE_WIDTH, PAGE_HEIGHT] });

    if (bgImage) {
        const pageWidth = doc.internal.pageSize.getWidth();
        const pageHeight = doc.internal.pageSize.getHeight();
        doc.addImage(bgImage, "PNG", 0, 0, pageWidth, pageHeight);
    }

    const TEXT_FONT = "times";
    const TEXT_SIZE = 13;

    // header
    doc.setFont(TEXT_FONT, "normal"); // also bold or italic
    doc.setFontSize(TEXT_SIZE);

    doc.text("asdasd", 20, 20, {});

    return doc.output("dataurlstring");
}
