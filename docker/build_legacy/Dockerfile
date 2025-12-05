FROM rust:1.89-bookworm

RUN apt-get update && apt-get install -y \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxinerama-dev \
    libxi-dev \
    libgl1-mesa-dev \
    libwayland-dev \
    libxkbcommon0 \
    libxkbcommon-x11-0 \
    libasound2-dev \
    libpulse-dev \
    libudev-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/

