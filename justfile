default:
    @just --list

# === Infrastructure ===

# Start dev infrastructure (db + minio)
infra:
    docker compose up -d db minio minio-setup

# Stop dev infrastructure
infra-down:
    docker compose down

# === Leptos Webapp ===

# Development server with hot reload
dev:
    cargo leptos watch

# Build release binary + WASM
build:
    cargo leptos build --release

# === Core Library ===

# Run tests on filmorator-core
test:
    cargo test -p filmorator-core

# === Linting ===

# Clippy on filmorator-core
clippy-core:
    cargo clippy -p filmorator-core -- -D warnings -W clippy::pedantic

# Clippy on filmorator-web (SSR)
clippy-ssr:
    cargo clippy -p filmorator-web --features ssr -- -D warnings

# Clippy on filmorator-web (hydrate/WASM)
clippy-hydrate:
    cargo clippy -p filmorator-web --target wasm32-unknown-unknown --features hydrate -- -D warnings

# Clippy all targets
clippy: clippy-core clippy-ssr clippy-hydrate

# === Formatting ===

# Format all code (rustfmt + leptosfmt)
fmt:
    cargo fmt --all
    leptosfmt filmorator-web/src

# Check formatting
fmt-check:
    cargo fmt --all -- --check
    leptosfmt --check filmorator-web/src

# Run all checks (format, clippy, test)
check: fmt-check clippy test

# === Database ===

# Run migrations (requires DATABASE_URL or running db container)
migrate-run:
    cargo sqlx migrate run

# Create new migration
migrate-new name:
    cargo sqlx migrate add {{name}}

# Generate sqlx offline data for CI
prepare-offline:
    cargo sqlx prepare --workspace

# === Docker ===

# Build Docker image for webapp
docker-build:
    docker build -t filmorator .

# === CI ===

# Local CI simulation
ci-check: fmt-check clippy test
    cargo leptos build --release

# === Legacy Reference ===

# Build legacy PoC (for reference only)
legacy-build:
    cd _legacy/filmorator-web && cargo build

# Run legacy PoC (for reference only)
legacy-run:
    cd _legacy/filmorator-web && cargo run
