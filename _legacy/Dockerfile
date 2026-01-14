FROM rust:1.88-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY filmorator-core/Cargo.toml filmorator-core/
COPY filmorator-web/Cargo.toml filmorator-web/

RUN mkdir -p filmorator-core/src filmorator-web/src && \
    echo "pub fn dummy() {}" > filmorator-core/src/lib.rs && \
    echo "fn main() {}" > filmorator-web/src/main.rs && \
    cargo build --release --package filmorator-web && \
    rm -rf filmorator-core/src filmorator-web/src

COPY filmorator-core/src filmorator-core/src
COPY filmorator-web/src filmorator-web/src
COPY migrations migrations

RUN touch filmorator-core/src/lib.rs filmorator-web/src/main.rs && \
    cargo build --release --package filmorator-web

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/filmorator-web /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations

WORKDIR /app
ENV RUST_LOG=filmorator_web=info

CMD ["filmorator-web"]
