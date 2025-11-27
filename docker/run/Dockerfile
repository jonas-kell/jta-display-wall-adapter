# ===== Stage 1: Build =====
FROM rust:1.89 AS builder

# Sound code from server needs sound dev libs to be installed, even if it will not work in the container anyway
RUN apt-get update && apt-get install -y \
    libasound2-dev \
    libpulse-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

ARG TARGETARCH
ENV CARGO_HOME=/usr/local/cargo-${TARGETARCH}
ENV CARGO_TARGET_DIR=/app/target-${TARGETARCH}

# Build the client in release mode
RUN --mount=type=cache,target=/usr/local/cargo-${TARGETARCH} \
    --mount=type=cache,target=/app/target-${TARGETARCH} \
    cargo build --release --bin jta-display-wall-adapter && mv /app/target-${TARGETARCH}/release/jta-display-wall-adapter /app

# ===== Stage 2: Runtime =====
FROM debian:trixie

# Install only runtime dependencies for GUI - display client has no sound. No sound runtime installed
RUN apt-get update && apt-get install -y \
    libx11-6 \
    libx11-xcb1 \
    libxcursor1 \
    libxrandr2 \
    libxinerama1 \
    libxi6 \
    libgl1-mesa-dri \
    libwayland-client0 \
    libxkbcommon0 \
    libxkbcommon-x11-0 \
    libxcb1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built executable from builder stage
COPY --from=builder /app/jta-display-wall-adapter /app/client
COPY README.md /usr/share/licenses
COPY LICENSE /usr/share/licenses
COPY THIRD_PARTY_LICENSES.md /usr/share/licenses
COPY THIRD-PARTY-LICENSES-RUST /usr/share/licenses
COPY THIRD-PARTY-LICENSES-JS /usr/share/licenses
