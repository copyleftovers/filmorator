# Filmorator - Implementation Plan

Film photography ranking webapp. Users anonymously compare photos in groups of 3-4, system derives rankings via Bradley-Terry model.

## Architecture

```
filmorator/
├── Cargo.toml                 # Workspace root
├── dist-workspace.toml        # cargo-dist config
├── filmorator-core/           # Pure domain logic (no I/O)
│   └── src/
│       ├── lib.rs
│       ├── models.rs          # Photo, Session, Matchup, ComparisonResult
│       ├── matchup.rs         # snic seed + dynamic selection
│       └── ranking.rs         # Bradley-Terry model
├── filmorator-web/            # Axum server
│   └── src/
│       ├── main.rs            # Router setup
│       ├── state.rs           # AppState (S3Client + DbPool)
│       ├── handlers/          # HTTP handlers
│       ├── db.rs              # sqlx queries
│       └── s3.rs              # S3 client
├── migrations/                # PostgreSQL migrations
├── justfile                   # Development tasks
├── Dockerfile
├── .github/
│   └── workflows/
│       ├── ci.yml             # Validation + release-plz PR
│       ├── release.yml        # cargo-dist binary builds
│       └── publish.yml        # crates.io publish
└── .pre-commit-config.yaml
```

## Core Data Models

```rust
pub struct Photo { id: Uuid, filename: String, width: u32, height: u32, file_hash: String, position: u32 }
pub struct Session { id: Uuid, created_at: DateTime, last_active_at: DateTime }
pub struct Matchup { id: Uuid, session_id: Uuid, photo_indices: Vec<u32>, is_seed: bool }
pub struct ComparisonResult { matchup_id: Uuid, session_id: Uuid, ranked_photo_indices: Vec<u32> }
pub struct PhotoRating { photo_idx: u32, strength: f64, uncertainty: f64 }  # Bradley-Terry params
```

## Database Schema (PostgreSQL)

```sql
CREATE TABLE photos (id UUID PK, filename TEXT, width INT, height INT, file_hash TEXT UNIQUE, position INT);
CREATE TABLE sessions (id UUID PK, created_at TIMESTAMPTZ, last_active_at TIMESTAMPTZ);
CREATE TABLE matchups (id UUID PK, session_id UUID FK, photo_indices INT[], is_seed BOOL DEFAULT false);
CREATE TABLE comparison_results (id UUID PK, matchup_id UUID FK, session_id UUID FK, ranked_photo_indices INT[]);
CREATE TABLE photo_ratings (session_id UUID FK, photo_idx INT, strength FLOAT8, uncertainty FLOAT8, PRIMARY KEY(session_id, photo_idx));
```

## API Routes

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/` | Landing page |
| `GET` | `/compare` | Comparison UI (HTML) |
| `POST` | `/api/matchup` | Get next matchup |
| `POST` | `/api/compare` | Submit ranking |
| `GET` | `/api/ranking` | Session ranking |
| `GET` | `/img/{tier}/{id}` | Presigned URL redirect |
| `GET` | `/admin` | Dashboard (token-protected) |
| `GET` | `/admin/session/:id` | User detail |
| `GET` | `/admin/aggregate` | Cross-user ranking |

## Core Algorithms

### Matchup Generation (Two-Phase)

**Phase 1: snic seed** (initial coverage)
- Use `snic_core::network::LocalMatchupsManager` with GBER decomposition
- Generates O(n log n) matchups covering all items efficiently
- Stored with `is_seed = true`
- Provides baseline for transitive inference

**Phase 2: Dynamic selection** (refinement)
- After seed exhausted, select matchups dynamically
- Prioritize pairs with high uncertainty (close ratings)
- Use information gain heuristic: compare items where outcome is most uncertain
- `uncertainty = 1 / (1 + |strength_a - strength_b|)`

### Ranking (Bradley-Terry Model)

Standard pairwise comparison model for ranking:
- Each photo has latent strength parameter `θ_i`
- P(i beats j) = `θ_i / (θ_i + θ_j)` = `1 / (1 + exp(-(log θ_i - log θ_j)))`
- Equivalent to logistic regression on pairwise outcomes
- Maximum likelihood estimation via iterative updates
- Handles multi-way comparisons by expanding to pairwise

**Update rule** (MM algorithm):
```
θ_i^{new} = wins_i / Σ_j (n_ij / (θ_i + θ_j))
```

**Uncertainty** from Fisher information matrix for confidence intervals.

## CI/CD (Three Workflows)

### 1. ci.yml - Validation + Release PR
```yaml
on: [push: main, pull_request: main]
env: { SQLX_OFFLINE: true }
jobs:
  validate:  # fmt, clippy (pedantic), build, test
  create-release-pr:  # release-plz/action@v0.5 (command: release-pr)
```

### 2. release.yml - cargo-dist Binary Builds
```yaml
on: push tags '**[0-9]+.[0-9]+.[0-9]+*'
jobs:
  plan → build-local-artifacts → build-global-artifacts → host → announce
# Cross-platform: aarch64-apple-darwin, aarch64-unknown-linux-gnu,
#                 x86_64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc
```

### 3. publish.yml - Crates.io Publish
```yaml
on: pull_request closed (merged to main)
jobs:
  publish-and-tag:  # release-plz/action@v0.5 (command: release)
```

### dist-workspace.toml
```toml
[workspace]
members = ["cargo:."]

[dist]
cargo-dist-version = "0.30.2"
ci = "github"
installers = []
targets = ["aarch64-apple-darwin", "aarch64-unknown-linux-gnu", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"]
```

### .pre-commit-config.yaml
- `doublify/pre-commit-rust`: fmt, cargo-check, clippy (pedantic)
- Standard hooks: trailing-whitespace, end-of-file-fixer

### justfile
```just
default:
    @just --list

dev:
    cargo run --bin filmorator-web

build:
    cargo build --release

test:
    cargo test --all-features

clippy:
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

fmt:
    cargo fmt --all

check: fmt clippy test

prepare-offline:
    cargo sqlx prepare --workspace

docker-build:
    docker build -t filmorator .

# Initialize cargo-dist
dist-init:
    cargo dist init
```

### Required Secrets
- `RELEASE_PLZ_TOKEN` - GitHub token with PR/contents write
- `CARGO_REGISTRY_TOKEN` - crates.io publish token

## Implementation Phases

### Phase 1: Foundation
1. Workspace setup (Cargo.toml, crates)
2. Core models
3. snic_core integration for seed matchups
4. Bradley-Terry ranking implementation
5. Basic Axum server skeleton
6. Compare page HTML (mobile-first, 3-photo grid)
7. CI setup (GitHub Actions, pre-commit, justfile)

### Phase 2: Persistence
1. PostgreSQL schema + migrations
2. sqlx queries + offline mode
3. Session management (cookie UUID)
4. Rating persistence

### Phase 3: S3 Integration
1. S3 client (reuse gallery-rs pattern)
2. 3-tier presigned URLs (thumb/preview/original)
3. Progressive loading JS

### Phase 4: Admin
1. Dashboard + session detail
2. Aggregate cross-user ranking
3. Photo management

### Phase 5: Polish
1. Docker multi-stage build
2. Error handling, logging
3. Rate limiting, health checks

## Key Dependencies

```toml
axum = "0.7"
tokio = { version = "1.42", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
aws-sdk-s3 = "1.63"
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.11", features = ["v4", "serde"] }
tracing = "0.1"
anyhow = "1.0"

# snic for matchup seeding
snic_core = { git = "https://github.com/ryzhakar/snic-rs", package = "snic_core" }
```

## Reference Files

**Web patterns:**
- `gallery-rs/gallery-web/src/handlers.rs` - Server-side HTML, presigned URLs
- `gallery-rs/gallery-core/src/s3.rs` - S3 client to reuse

**Matchup generation:**
- `snic-rs/snic_core/src/network/matchup.rs` - LocalMatchupsManager for seed generation
- `snic-rs/snic_core/src/gber/mod.rs` - GBER decomposition

**CI/CD (copy these):**
- `monobank-sync/.github/workflows/ci.yml` - Validation + release-plz PR
- `monobank-sync/.github/workflows/release.yml` - cargo-dist binary builds
- `monobank-sync/.github/workflows/publish.yml` - crates.io publish
- `monobank-sync/dist-workspace.toml` - cargo-dist config
- `monobank-sync/justfile` - justfile patterns
- `monobank-sync/.pre-commit-config.yaml` - pre-commit hooks

## Environment

```bash
DATABASE_URL=postgresql://...
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
FILMORATOR_BUCKET=filmorator-photos
AWS_ENDPOINT_URL=http://minio:9000  # for dev
PORT=3000
ADMIN_TOKEN=secret
RUST_LOG=filmorator_web=info
```
