import { DayTime, RaceWind } from "./interfaceInbound";

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
