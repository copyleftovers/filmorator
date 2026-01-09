default:
    @just --list

# Run development server
dev:
    cargo run --bin filmorator-web

# Build release binary
build:
    cargo build --release

# Run all tests
test:
    cargo test --all-features

# Run clippy with pedantic warnings
clippy:
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

# Format all code
fmt:
    cargo fmt --all

# Run all checks (format, clippy, test)
check: fmt clippy test

# Generate sqlx offline data for CI
prepare-offline:
    cargo sqlx prepare --workspace

# Build Docker image
docker-build:
    docker build -t filmorator .

# Initialize cargo-dist (run once for initial setup)
dist-init:
    cargo dist init

# Run local CI simulation
ci-check: fmt
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
    cargo build
    cargo test

# Database migrations (requires DATABASE_URL)
migrate-run:
    cargo sqlx migrate run

migrate-new name:
    cargo sqlx migrate add {{name}}

migrate-revert:
    cargo sqlx migrate revert
