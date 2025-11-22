FROM rust:1.89

RUN apt-get update && \
    apt-get install -y libsqlite3-dev sqlite3 pkg-config && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install diesel_cli --no-default-features --features sqlite

WORKDIR /app

ENTRYPOINT ["diesel"]