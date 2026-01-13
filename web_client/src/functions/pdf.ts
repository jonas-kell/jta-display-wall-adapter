import { jsPDF } from "jspdf";
import { PDFConfigurationSetting, RaceTime } from "./../generated/interface";
import { AthletePrintData } from "./sharedAthleteTypes";
import { raceTimeStringRepr, subtractRaceTimes } from "./representation";

export enum PDFConfigurationContentReferenceReference {
    Bib = "Bib",
    Name = "Name",
    FirstName = "FirstName",
    LastName = "LastName",
    HasRound1 = "HasRound1",
    HasRound2 = "HasRound2",
    HasRound3 = "HasRound3",
    HasRound4 = "HasRound4",
    HasRound5 = "HasRound5",
    HasRound6 = "HasRound6",
    TimeRound1 = "TimeRound1",
    TimeRound2 = "TimeRound2",
    TimeRound3 = "TimeRound3",
    TimeRound4 = "TimeRound4",
    TimeRound5 = "TimeRound5",
    TimeRound6 = "TimeRound6",
    TotalTimeRound1 = "TotalTimeRound1",
    TotalTimeRound2 = "TotalTimeRound2",
    TotalTimeRound3 = "TotalTimeRound3",
    TotalTimeRound4 = "TotalTimeRound4",
    TotalTimeRound5 = "TotalTimeRound5",
    TotalTimeRound6 = "TotalTimeRound6",
    FinalTime = "FinalTime",
    SpkTime = "SpkTime",
    SpkGuess = "SpkGuess",
}

export function generatePDF(
    download: boolean,
    landscape: boolean,
    bgImage: string | null,
    settings: PDFConfigurationSetting[],
    data: AthletePrintData[] | null
): string | null {
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

    const TEXT_FONT = "times";
    const BG_ALIAS = "background-image";
    let bgAdded = false;

    function page(athlete: AthletePrintData | null) {
        if (bgImage) {
            const pageWidth = doc.internal.pageSize.getWidth();
            const pageHeight = doc.internal.pageSize.getHeight();
            if (!bgAdded) {
                doc.addImage(bgImage, "PNG", 0, 0, pageWidth, pageHeight, BG_ALIAS);
            } else {
                doc.addImage(BG_ALIAS, "PNG", 0, 0, pageWidth, pageHeight);
            }
        }

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
                    text = set.content.data.text;
                    break;
                case "PDFConfigurationContentReference":
                    if (athlete) {
                        const alt = set.content.data.reference_content ?? "";
                        const altTextIfRound = (i: number) => {
                            if (athlete.roundTimes.length + 1 > i) {
                                return alt;
                            } else {
                                return "";
                            }
                        };
                        const timeTextOfRoundIfRound = (i: number) => {
                            if (athlete.roundTimes.length + 1 > i) {
                                return raceTimeStringRepr(athlete.roundTimes[i - 1], false, true, 2);
                            } else {
                                return "";
                            }
                        };
                        const timeTextOfOnlyRoundIfRound = (i: number) => {
                            if (athlete.roundTimes.length + 1 > i) {
                                const relevantRoundTime = athlete.roundTimes[i - 1];
                                const previousRoundTime =
                                    i - 2 >= 0
                                        ? athlete.roundTimes[i - 2]
                                        : ({
                                              hours: null,
                                              minutes: null,
                                              seconds: 0,
                                              hundrets: null,
                                              ten_thousands: null,
                                              tenths: null,
                                              thousands: null,
                                          } as RaceTime);

                                return raceTimeStringRepr(
                                    subtractRaceTimes(relevantRoundTime, previousRoundTime),
                                    false,
                                    true,
                                    2
                                );
                            } else {
                                return "";
                            }
                        };
                        switch (set.content.data.reference) {
                            case PDFConfigurationContentReferenceReference.Bib:
                                text = String(athlete.bib);
                                break;
                            case PDFConfigurationContentReferenceReference.Name:
                                text = String(athlete.firstName + " " + athlete.lastName);
                                break;
                            case PDFConfigurationContentReferenceReference.FirstName:
                                text = String(athlete.firstName);
                                break;
                            case PDFConfigurationContentReferenceReference.LastName:
                                text = String(athlete.lastName);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound1:
                                text = altTextIfRound(1);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound2:
                                text = altTextIfRound(2);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound3:
                                text = altTextIfRound(3);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound4:
                                text = altTextIfRound(4);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound5:
                                text = altTextIfRound(5);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound6:
                                text = altTextIfRound(6);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound1:
                                text = timeTextOfRoundIfRound(1);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound2:
                                text = timeTextOfRoundIfRound(2);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound3:
                                text = timeTextOfRoundIfRound(3);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound4:
                                text = timeTextOfRoundIfRound(4);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound5:
                                text = timeTextOfRoundIfRound(5);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound6:
                                text = timeTextOfRoundIfRound(6);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound1:
                                text = timeTextOfOnlyRoundIfRound(1);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound2:
                                text = timeTextOfOnlyRoundIfRound(2);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound3:
                                text = timeTextOfOnlyRoundIfRound(3);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound4:
                                text = timeTextOfOnlyRoundIfRound(4);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound5:
                                text = timeTextOfOnlyRoundIfRound(5);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound6:
                                text = timeTextOfOnlyRoundIfRound(6);
                                break;
                            case PDFConfigurationContentReferenceReference.FinalTime:
                                if (athlete.roundTimes.length > 0) {
                                    text = timeTextOfRoundIfRound(athlete.roundTimes.length);
                                }
                                break;
                            case PDFConfigurationContentReferenceReference.SpkGuess:
                            case PDFConfigurationContentReferenceReference.SpkTime:
                                // TODO addthese cases
                                text = "";
                                break;
                            default:
                                break;
                        }
                    } else {
                        text = "ref";
                    }
                    break;
                default:
                    break;
            }
            doc.text(text, set.pos_x, set.pos_y, {
                align: set.centered ? "center" : "left",
            });
        });
    }

    if (data && data.length > 0) {
        for (let index = 0; index < data.length; index++) {
            const athlete = data[index];
            page(athlete);
            if (index + 1 < data.length) {
                doc.addPage();
            }
        }
    } else {
        page(null);
    }

    if (download) {
        doc.save("bib-certificate.pdf");
    } else {
        return doc.output("dataurlstring");
    }

    return null;
}
