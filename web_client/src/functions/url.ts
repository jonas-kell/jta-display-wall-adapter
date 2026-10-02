export function getNonLocalDomainOrIp(): string | null {
    const name = window.location.hostname;

    if (name == "localhost") {
        return null;
    }

    if (name == "127.0.0.1") {
        return null;
    }

    if (name == "0.0.0.0") {
        return null;
    }

    return name;
}
