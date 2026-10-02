import { jsPDF } from "jspdf";
import { PDFConfigurationSetting, RaceTime } from "./../generated/interface";
import { AthletePrintData } from "./sharedAthleteTypes";
import { raceTimeStringRepr, subtractRaceTimes } from "./representation";
import { getNonLocalDomainOrIp } from "./url";
import { restURL } from "./environment";

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

export enum PDFConfigurationContentImageReferenceReference {
    BibAruco1 = "BibAruco1",
    BibAruco2 = "BibAruco2",
}

export async function generatePDF(
    download: boolean,
    landscape: boolean,
    bgImage: string | null,
    settings: PDFConfigurationSetting[],
    data: AthletePrintData[] | null,
): Promise<string | null> {
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

    async function page(athlete: AthletePrintData | null) {
        if (bgImage) {
            const pageWidth = doc.internal.pageSize.getWidth();
            const pageHeight = doc.internal.pageSize.getHeight();
            if (!bgAdded) {
                doc.addImage(bgImage, "PNG", 0, 0, pageWidth, pageHeight, BG_ALIAS);
            } else {
                doc.addImage(BG_ALIAS, "PNG", 0, 0, pageWidth, pageHeight);
            }
        }

        for (const set of settings) {
            doc.setFont(TEXT_FONT, "normal"); // also bold or italic
            if (set.bold) {
                doc.setFont(TEXT_FONT, "bold");
            }
            if (set.italic) {
                doc.setFont(TEXT_FONT, "italic");
            }
            doc.setFontSize(set.size);
            switch (set.content.type) {
                case "PDFConfigurationContentText":
                    let textfix = set.content.data.text;
                    doc.text(textfix, set.pos_x, set.pos_y, {
                        align: set.centered ? "center" : "left",
                    });
                    break;
                case "PDFConfigurationContentReference":
                    let textref = "";
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
                                    2,
                                );
                            } else {
                                return "";
                            }
                        };
                        switch (set.content.data.reference) {
                            case PDFConfigurationContentReferenceReference.Bib:
                                textref = String(athlete.bib);
                                break;
                            case PDFConfigurationContentReferenceReference.Name:
                                textref = String(athlete.firstName + " " + athlete.lastName);
                                break;
                            case PDFConfigurationContentReferenceReference.FirstName:
                                textref = String(athlete.firstName);
                                break;
                            case PDFConfigurationContentReferenceReference.LastName:
                                textref = String(athlete.lastName);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound1:
                                textref = altTextIfRound(1);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound2:
                                textref = altTextIfRound(2);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound3:
                                textref = altTextIfRound(3);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound4:
                                textref = altTextIfRound(4);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound5:
                                textref = altTextIfRound(5);
                                break;
                            case PDFConfigurationContentReferenceReference.HasRound6:
                                textref = altTextIfRound(6);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound1:
                                textref = timeTextOfRoundIfRound(1);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound2:
                                textref = timeTextOfRoundIfRound(2);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound3:
                                textref = timeTextOfRoundIfRound(3);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound4:
                                textref = timeTextOfRoundIfRound(4);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound5:
                                textref = timeTextOfRoundIfRound(5);
                                break;
                            case PDFConfigurationContentReferenceReference.TotalTimeRound6:
                                textref = timeTextOfRoundIfRound(6);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound1:
                                textref = timeTextOfOnlyRoundIfRound(1);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound2:
                                textref = timeTextOfOnlyRoundIfRound(2);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound3:
                                textref = timeTextOfOnlyRoundIfRound(3);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound4:
                                textref = timeTextOfOnlyRoundIfRound(4);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound5:
                                textref = timeTextOfOnlyRoundIfRound(5);
                                break;
                            case PDFConfigurationContentReferenceReference.TimeRound6:
                                textref = timeTextOfOnlyRoundIfRound(6);
                                break;
                            case PDFConfigurationContentReferenceReference.FinalTime:
                                if (athlete.roundTimes.length > 0) {
                                    textref = timeTextOfRoundIfRound(athlete.roundTimes.length);
                                }
                                break;
                            case PDFConfigurationContentReferenceReference.SpkGuess:
                            case PDFConfigurationContentReferenceReference.SpkTime:
                                // TODO add these cases
                                textref = "";
                                break;
                            default:
                                break;
                        }
                    } else {
                        textref = "ref";
                    }
                    doc.text(textref, set.pos_x, set.pos_y, {
                        align: set.centered ? "center" : "left",
                    });
                    break;
                case "PDFConfigurationContentImageReference":
                    const ARUCO_RESOLUTION = 200;
                    let width = 0.0;
                    let height = 0.0;
                    let url = "";
                    const BASE_REST_URL = restURL(getNonLocalDomainOrIp());

                    function calculateArucoIndex(athlete: AthletePrintData | null, first: boolean) {
                        if (athlete) {
                            return parseInt(String(athlete.bib)) + (first ? 0 : 500);
                        } else {
                            return 10000;
                        }
                    }

                    switch (set.content.data.reference) {
                        case PDFConfigurationContentImageReferenceReference.BibAruco1:
                            width = set.size;
                            height = set.size;
                            url =
                                BASE_REST_URL +
                                "aruco/" +
                                calculateArucoIndex(athlete, true) +
                                "/" +
                                ARUCO_RESOLUTION +
                                "/mrk.png";
                            break;
                        case PDFConfigurationContentImageReferenceReference.BibAruco2:
                            width = set.size;
                            height = set.size;
                            url =
                                BASE_REST_URL +
                                "aruco/" +
                                calculateArucoIndex(athlete, false) +
                                "/" +
                                ARUCO_RESOLUTION +
                                "/mrk.png";
                            break;
                        default:
                            break;
                    }

                    if (url != "") {
                        let uri = await fetchAsDataUri(url);
                        doc.addImage(uri, set.pos_x, set.pos_y, width, height);
                    }
                    break;
                default:
                    break;
            }
        }
    }

    if (data && data.length > 0) {
        for (let index = 0; index < data.length; index++) {
            const athlete = data[index];
            await page(athlete);
            if (index + 1 < data.length) {
                doc.addPage();
            }
        }
    } else {
        await page(null);
    }

    if (download) {
        doc.save("bib-certificate.pdf");
    } else {
        return doc.output("dataurlstring");
    }

    return null;
}

async function fetchAsDataUri(url: string): Promise<string> {
    const response = await fetch(url);

    if (!response.ok) {
        throw new Error(`Failed to fetch ${url}: ${response.status} ${response.statusText}`);
    }

    const contentType = response.headers.get("content-type") ?? "application/octet-stream";

    const bytes = new Uint8Array(await response.arrayBuffer());

    let binary = "";
    for (let i = 0; i < bytes.length; i += 0x8000) {
        binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
    }

    return `data:${contentType};base64,${btoa(binary)}`;
}
