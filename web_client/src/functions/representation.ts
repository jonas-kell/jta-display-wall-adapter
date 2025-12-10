import { DayTime } from "./interfaceInbound";

function zeroPad(value: number, length: number): string {
    return String(value).padStart(length, "0");
}

export function dayTimeStringRepr(dt: DayTime): string {
    return `${zeroPad(dt.hours, 2)}:${zeroPad(dt.minutes, 2)}:${zeroPad(dt.seconds, 2)}`;
}
