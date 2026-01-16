export function wsURL(overwriteIp: string | null = null) {
    let ip = String(import.meta.env.VITE_WS_IP);
    const port = String(import.meta.env.VITE_WS_PORT);

    if (overwriteIp != null) {
        ip = overwriteIp;
    }

    return `ws://${ip}:${port}/ws/`;
}
