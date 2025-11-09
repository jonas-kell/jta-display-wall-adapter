# ===== Stage 1: Build =====
FROM rust:1.89 AS builder

WORKDIR /app
COPY . .

# Build the client in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin jta-display-wall-adapter

# ===== Stage 2: Runtime =====
FROM debian:trixie

# Install only runtime dependencies for GUI
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
COPY --from=builder /app/target/release/jta-display-wall-adapter /app/client
