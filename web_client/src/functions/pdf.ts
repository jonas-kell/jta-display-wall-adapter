import { jsPDF } from "jspdf";
import { PDFConfigurationSetting } from "./interfaceShared";

export function generatePDF(landscape: boolean, bgImage: string | null, settings: PDFConfigurationSetting[]) {
    // A4 page portrait
    let PAGE_HEIGHT = 297;
    let PAGE_WIDTH = 210;
    let PAGE_ORIENTATION = "p" as "l" | "p";

    // A5 page landscape
    if (landscape) {
        PAGE_HEIGHT = 148;
        PAGE_WIDTH = 210;
        PAGE_ORIENTATION = "l";
    }

    const doc = new jsPDF({ orientation: PAGE_ORIENTATION, unit: "mm", format: [PAGE_WIDTH, PAGE_HEIGHT] });

    if (bgImage) {
        const pageWidth = doc.internal.pageSize.getWidth();
        const pageHeight = doc.internal.pageSize.getHeight();
        doc.addImage(bgImage, "PNG", 0, 0, pageWidth, pageHeight);
    }

    const TEXT_FONT = "times";

    // header
    settings.forEach((set) => {
        doc.setFont(TEXT_FONT, "normal"); // also bold or italic
        if (set.bold) {
            doc.setFont(TEXT_FONT, "bold");
        }
        if (set.italic) {
            doc.setFont(TEXT_FONT, "italic");
        }
        doc.setFontSize(set.size);
        let text = "";
        switch (set.content.type) {
            case "PDFConfigurationContentText":
                text = set.content.text;
                break;
            case "PDFConfigurationContentReference":
                text = "ref";
                break;
            default:
                break;
        }
        doc.text(text, set.pos_x, set.pos_y, {});
    });

    return doc.output("dataurlstring");
}
