import { DayTime, RaceTime, RaceWind } from "./interfaceInbound";

function zeroPad(value: number, length: number): string {
    return String(value).padStart(length, "0");
}

export function dayTimeStringRepr(dt: DayTime): string {
    return `${zeroPad(dt.hours, 2)}:${zeroPad(dt.minutes, 2)}:${zeroPad(dt.seconds, 2)}`;
}

export function numberFromWind(wind: RaceWind): number {
    let res = 0;

    res += wind.whole_number_part;
    res += wind.fraction_part / 10;

    if (!wind.back_wind) {
        res *= -1;
    }

    return res;
}

export function windStringRepr(wind: RaceWind): string {
    return `${numberFromWind(wind).toFixed(1)}`;
}

export function imageURLfromBMPBytes(data: ArrayBuffer) {
    const uint8 = new Uint8Array(data);
    const blob = new Blob([uint8], { type: "image/bmp" });
    return URL.createObjectURL(blob);
}

export function imageURLfromBMPBytesArray(data: number[]) {
    const uint8 = new Uint8Array(data);
    const blob = new Blob([uint8], { type: "image/bmp" });
    return URL.createObjectURL(blob);
}

export function numberFromRaceTime(rt: RaceTime): number {
    let secs =
        (rt.hours ?? 0) * 3600 +
        (rt.minutes ?? 0) * 60 +
        rt.seconds +
        (rt.tenths ?? 0) * 0.1 +
        (rt.hundrets ?? 0) * 0.01 +
        (rt.thousands ?? 0) * 0.001 +
        (rt.ten_thousands ?? 0) * 0.0001;

    return Math.round(secs * 100) / 100;
}
