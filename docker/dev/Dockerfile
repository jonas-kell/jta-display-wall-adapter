FROM rust:1.89

RUN cargo install cargo-watch
RUN rustup component add rustfmt

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
    libasound2 \
    libasound2-dev \
    libpulse0 \
    libpulse-dev \
    libpulse-mainloop-glib0 \
    alsa-utils \
    libasound2-plugins \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN cat <<'EOF' > /etc/asound.conf
pcm.!default {
    type pulse
    fallback "sysdefault"
    hint {
        show on
        description "PulseAudio Sound Server"
    }
}

ctl.!default {
    type pulse
}
EOF

RUN cargo install watchexec-cli

WORKDIR /app/

