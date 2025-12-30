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

export function raceTimeStringRepr(
    rt: RaceTime,
    displayHoursIfZero: boolean,
    displayMinutesIfZero: boolean,
    fractionDigits: number
): string {
    const parts: string[] = [];

    const hours = rt.hours !== null ? rt.hours : displayHoursIfZero ? 0 : null;

    const minutes = rt.minutes !== null ? rt.minutes : displayMinutesIfZero || hours !== null ? 0 : null;

    if (hours !== null) {
        parts.push(hours.toString());
        parts.push(minutes!.toString().padStart(2, "0"));
    } else if (minutes !== null) {
        parts.push(minutes.toString());
    }

    parts.push(rt.seconds.toString().padStart(parts.length > 0 ? 2 : 1, "0"));

    if (fractionDigits > 0) {
        const availableDigits = [rt.tenths, rt.hundrets, rt.thousands, rt.ten_thousands];

        const fraction = availableDigits
            .slice(0, fractionDigits)
            .map((d) => (d ?? 0).toString())
            .join("");

        return `${parts.join(":")}.${fraction}`;
    }

    return parts.join(":");
}

function raceTimeFromNumber(value: number): RaceTime {
    // Round to 4 decimal places to match precision in numberFromRaceTime
    value = Math.round(value * 10000) / 10000;

    const hours = Math.floor(value / 3600);
    value -= hours * 3600;

    const minutes = Math.floor(value / 60);
    value -= minutes * 60;

    const seconds = Math.floor(value);
    let fraction = value - seconds;

    const tenths = Math.floor(fraction * 10);
    fraction -= tenths * 0.1;

    const hundrets = Math.floor(fraction * 100);
    fraction -= hundrets * 0.01;

    const thousands = Math.floor(fraction * 1000);
    fraction -= thousands * 0.001;

    const ten_thousands = Math.floor(fraction * 10000);

    return {
        hours: hours,
        minutes: minutes,
        seconds,
        tenths: tenths,
        hundrets: hundrets,
        thousands: thousands,
        ten_thousands: ten_thousands,
    };
}

export function subtractRaceTimes(a: RaceTime, b: RaceTime): RaceTime {
    const time = numberFromRaceTime(a) - numberFromRaceTime(b);

    return raceTimeFromNumber(time);
}
