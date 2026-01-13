# Leptos Migration Synthesis

**Generated**: 2026-01-13
**Validated**: 2026-01-13 (constitutional review applied)
**Track B Synthesized**: 2026-01-13 (compiler-driven workflow)
**Source Repos**: 8 cloned, 11 parallel investigation agents (5 Track A + 6 Track B)

## Constitutional Constraints Applied

- **Correct By Construction**: Invalid states unrepresentable at compile time
- **Simple Made Easy**: Favor untangled solutions over familiar ones

## Validation Status

### Track A: Core Framework

| Section | Status | Notes |
|---------|--------|-------|
| Tailwind integration | ✅ Verified | `cargo-leptos/src/ext/exe.rs:352` confirms standalone binary |
| S3 client injection | ✅ Verified | `leptos/integrations/axum/src/lib.rs:1699` |
| leptos-image incompatibility | ✅ Verified | Requires local filesystem, not S3 |
| RankState pattern | ✅ Verified | Leptos autofixer passes |
| Cargo.toml template | ⚠️ Corrected | Removed non-existent `testing` feature |
| ProgressiveImage | ⚠️ Corrected | Changed to use `Resource` (untangled) |

### Track B: Compiler-Driven Workflow

| Section | Status | Notes |
|---------|--------|-------|
| Dual-target clippy | ✅ Solved | `cargo-all-features` handles automatically |
| bacon + cargo-leptos | ⚠️ Incompatible | Use separately for different purposes |
| leptosfmt pre-commit | ✅ Solved | Custom local hook with `--check` |
| CI pipeline | ✅ Solved | Dual clippy jobs, leptosfmt-action |
| cargo-dist + Leptos | ⚠️ Hybrid | cargo-dist for CLI, Docker for webapp |
| release-plz workspace | ✅ Solved | `publish = false` for webapp |

## Hard Constraints

- NO JavaScript/TypeScript tooling (npm, node, bun, deno)
- Pure Rust toolchain + standalone binaries only

---

## Critical Decisions Resolved

### 1. Tailwind Integration: SOLVED

**Solution**: cargo-leptos auto-downloads Tailwind standalone binary

```toml
# Cargo.toml
[package.metadata.leptos]
tailwind-input-file = "style/tailwind.css"
# tailwind-config-file optional in v4
```

```css
/* style/tailwind.css */
@import "tailwindcss";
```

- Binary cached in `~/Library/Caches/cargo-leptos/`
- Tailwind v4 CSS-only syntax (no JS config required)
- NO npm/node needed

### 2. Dual S3 Client Injection: SOLVED

**Solution**: `FromRef` derive + `leptos_routes_with_context`

```rust
#[derive(FromRef, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub db_pool: PgPool,
    pub s3_internal: S3Client,   // minio:9000
    pub s3_presign: S3Client,    // localhost:9000
}

// In main.rs:
let app = Router::new()
    .leptos_routes_with_context(
        &app_state,
        routes,
        {
            let state = app_state.clone();
            move || {
                provide_context(state.db_pool.clone());
                provide_context(state.s3_internal.clone());
                provide_context(state.s3_presign.clone());
            }
        },
        || shell(leptos_options.clone()),
    )
    .with_state(app_state);

// In server functions:
#[server]
async fn get_matchup(campaign_id: String) -> Result<Matchup, ServerFnError> {
    let pool = expect_context::<PgPool>();
    let s3 = expect_context::<S3Client>(); // presign client
    // ...
}
```

### 3. Project Structure: SOLVED

**Recommended**: Workspace with separate crates (supports future CLI)

```
filmorator/
├── Cargo.toml                 # [[workspace.metadata.leptos]] array syntax
├── crates/
│   ├── filmorator-core/       # Shared types (lib) - no Leptos deps
│   ├── filmorator-web/        # Leptos webapp
│   │   ├── Cargo.toml         # bin-package + lib-package
│   │   ├── src/
│   │   │   ├── lib.rs         # App (WASM) - lib-package
│   │   │   └── main.rs        # Server (Axum) - bin-package
│   │   └── style/
│   │       └── tailwind.css
│   └── filmorator-cli/        # Future CLI (no Leptos)
```

**Why workspace over single-crate**: VISION.md specifies CLI + webapp sharing types.
Per Simple Made Easy: workspace is inherent complexity (CLI requirement), not incidental.

### 4. Image Handling: SOLVED

**Decision**: Hand-roll progressive loading (leptos-image incompatible with S3)

**Why leptos-image doesn't work**:
- Requires local filesystem images
- Does server-side optimization
- Incompatible with presigned URLs

**Pattern** (using Resource - untangled async):
```rust
#[component]
pub fn ProgressiveImage(
    campaign_id: String,
    filename: String,
) -> impl IntoView {
    // Resource: declarative async data fetching (Simple Made Easy)
    let urls = Resource::new(
        move || (campaign_id.clone(), filename.clone()),
        |(cid, fname)| async move { get_image_urls(cid, fname).await.ok() }
    );

    view! {
        <Suspense fallback=|| view! { <div class="animate-pulse bg-gray-200" /> }>
            {move || urls.get().flatten().map(|u| view! {
                <img src=u.thumb class="thumb" />
                <LazyLoad>
                    <img src=u.preview class="preview" />
                </LazyLoad>
            })}
        </Suspense>
    }
}
```

**Why Resource over spawn_local+effect**:
- Resource is a value (Simple Made Easy principle)
- spawn_local in effect entangles lifecycle with async execution
- Resource integrates with Suspense for loading states
- Lazy loading via `leptos-use`'s `use_intersection_observer`

### 5. Ranking State: SOLVED

**Pattern**: Type-safe enum prevents invalid states

```rust
pub enum RankState {
    Empty,
    One(usize),
    Two(usize, usize),
    Complete(usize, usize, usize),
}

impl RankState {
    pub fn toggle(self, position: usize) -> Self {
        match self {
            RankState::Empty => RankState::One(position),
            RankState::One(p1) => {
                if p1 == position { RankState::Empty }
                else { RankState::Two(p1, position) }
            }
            // ... etc
        }
    }
}
```

### 6. Component Props: SOLVED

**Pattern** from Thaw UI:
```rust
#[component]
fn PhotoCard(
    photo: PhotoData,
    #[prop(optional)] rank: Option<Rank>,
    #[prop(optional)] on_click: Option<Box<dyn Fn() + 'static>>,
) -> impl IntoView {
    view! {
        <div
            class:rank-gold=move || rank == Some(Rank::Gold)
            class:rank-silver=move || rank == Some(Rank::Silver)
            on:click=move |_| if let Some(cb) = &on_click { cb() }
        >
            <img src=photo.thumb_url />
        </div>
    }
}
```

---

## Track B: Compiler-Driven Workflow Decisions

### 7. Dual-Target Linting: SOLVED

**Problem**: SSR and hydrate features are mutually exclusive. `--all-features` fails.

**Solution**: `cargo-all-features` (custom fork) handles feature matrix automatically.

```bash
# Installation
cargo install --git https://github.com/sabify/cargo-all-features --branch arbitrary-command-support

# Usage - runs clippy on all valid feature combinations
cargo all-features clippy --no-deps -- -D warnings
```

**Why this works**:
- Automatically detects mutually exclusive features
- Runs clippy for each valid combination (ssr alone, hydrate alone)
- No manual target specification needed
- Used by Leptos main repo

**Constitutional compliance**: Simple Made Easy - single command replaces manual feature matrix.

### 8. bacon + cargo-leptos: INCOMPATIBLE

**Problem**: bacon requires `--message-format=json`. cargo-leptos doesn't support it.

**Solution**: Use them for different purposes.

| Tool | Purpose |
|------|---------|
| `cargo leptos watch` | Development server (hot reload, WASM + server) |
| `bacon` | Clippy/check jobs only (not for cargo-leptos) |

```toml
# bacon.toml - for linting only
[jobs.clippy-all]
command = ["cargo", "all-features", "clippy", "--no-deps", "--", "-D", "warnings"]

[jobs.check-ssr]
command = ["cargo", "check", "--features", "ssr"]

[jobs.check-hydrate]
command = ["cargo", "check", "--target", "wasm32-unknown-unknown", "--features", "hydrate"]
```

**Why not wrap cargo-leptos**:
- cargo-leptos orchestrates multiple parallel builds (server + WASM + assets + style)
- bacon expects single cargo command with JSON output
- cargo-leptos uses structured logging (tracing), not cargo's format

### 9. leptosfmt Pre-commit: SOLVED

**Problem**: No native pre-commit hook exists for leptosfmt.

**Solution**: Custom local hook.

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/doublify/pre-commit-rust
    rev: v1.0
    hooks:
      - id: fmt
      - id: clippy
        args: [--fix, --allow-staged, --, -W, "clippy::pedantic"]

  - repo: local
    hooks:
      - id: leptosfmt
        name: leptosfmt
        description: Format Leptos view! macros
        entry: leptosfmt --check
        language: system
        files: \.rs$
        exclude: ^target/
```

**Order**: leptosfmt runs BEFORE rustfmt (processes `view!` macros first).

**Installation**: `cargo install leptosfmt` or use standalone binary from releases.

### 10. CI Pipeline: SOLVED

**Pattern**: Dual clippy jobs + leptosfmt-action.

```yaml
# .github/workflows/ci.yml
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
          components: clippy, rustfmt

      - name: Install tools
        run: |
          cargo binstall cargo-leptos --locked --no-confirm
          cargo install --git https://github.com/sabify/cargo-all-features --branch arbitrary-command-support

      - uses: Swatinem/rust-cache@v2

      # Formatting
      - run: cargo fmt --all -- --check
      - uses: LesnyRumcajs/leptosfmt-action@main
        with:
          args: --check

      # Linting (all feature combinations)
      - run: cargo all-features clippy --no-deps -- -D warnings

      # Build
      - run: cargo leptos build --release

      # Test
      - run: cargo test --features ssr
```

**Key insight**: `cargo all-features clippy` replaces separate ssr/hydrate jobs.

### 11. Release Strategy: HYBRID

**Problem**: cargo-dist builds native binaries only. Leptos produces server binary + WASM bundle.

**Solution**: Hybrid release strategy.

| Package | Release Method | Publish |
|---------|---------------|---------|
| `filmorator-core` | cargo-dist | crates.io ✅ |
| `filmorator-cli` | cargo-dist | crates.io ✅ |
| `filmorator-web` | Docker | Container registry ✅, crates.io ❌ |

```toml
# dist-workspace.toml
[workspace]
members = ["cargo:filmorator-core", "cargo:filmorator-cli"]
# Note: filmorator-web NOT in cargo-dist - uses Docker
```

```toml
# release-plz.toml
[workspace]
changelog_update = true
git_release_enable = true

[[package]]
name = "filmorator-core"
version_group = "published"

[[package]]
name = "filmorator-cli"
version_group = "published"

[[package]]
name = "filmorator-web"
publish = false
version_group = "webapp"
```

**Why hybrid**:
- Leptos ecosystem standardizes on Docker for webapp releases
- No examples of cargo-dist + Leptos in production
- cargo-dist `extra-artifacts` doesn't support globs (`target/site/**/*` fails)
- Inherent complexity (dual-target), not incidental

**Constitutional compliance**: Simple Made Easy - use each tool for what it's designed for, don't force cargo-dist to do something it wasn't built for.

---

## Configuration Reference

### Cargo.toml Template

```toml
[package]
name = "filmorator-web"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
leptos = { version = "0.8.2" }  # No "testing" feature exists
leptos_router = { version = "0.8.2" }
leptos_meta = { version = "0.8.2" }
leptos_axum = { version = "0.8.2", optional = true }
tokio = { version = "1", features = ["rt-multi-thread"], optional = true }
axum = { version = "0.8", optional = true }
wasm-bindgen = { version = "0.2.106", optional = true }
console_error_panic_hook = { version = "0.1", optional = true }

[features]
default = []
hydrate = ["leptos/hydrate", "dep:wasm-bindgen", "dep:console_error_panic_hook"]
ssr = ["leptos/ssr", "leptos_router/ssr", "leptos_meta/ssr", "dep:axum", "dep:tokio", "dep:leptos_axum"]

[profile.wasm-release]
inherits = "release"
opt-level = 'z'
lto = true
codegen-units = 1
panic = "abort"

[package.metadata.leptos]
output-name = "filmorator"
site-root = "target/site"
site-pkg-dir = "pkg"
tailwind-input-file = "style/tailwind.css"
assets-dir = "public"
site-addr = "127.0.0.1:3000"
reload-port = 3001
env = "DEV"
bin-features = ["ssr"]
bin-default-features = false
lib-features = ["hydrate"]
lib-default-features = false
lib-profile-release = "wasm-release"
```

### justfile Template

```just
default:
    @just --list

# Development server with hot reload
dev:
    cargo leptos watch

# Build release binary + WASM
build:
    cargo leptos build --release

# Run all tests
test:
    cargo test --features ssr

# Lint all feature combinations
lint:
    cargo all-features clippy --no-deps -- -D warnings

# Format all code (rustfmt + leptosfmt)
fmt:
    cargo fmt --all
    leptosfmt .

# Check formatting without changes
fmt-check:
    cargo fmt --all -- --check
    leptosfmt --check .

# Run all checks (format, lint, test)
check: fmt-check lint test

# Generate sqlx offline data for CI
prepare-offline:
    cargo sqlx prepare --workspace

# Build Docker image for webapp
docker-build:
    docker build -t filmorator .

# Local CI simulation
ci-check: fmt-check
    cargo all-features clippy --no-deps -- -D warnings
    cargo leptos build --release
    cargo test --features ssr

# Database migrations
migrate-run:
    cargo sqlx migrate run

migrate-new name:
    cargo sqlx migrate add {{name}}
```

### bacon.toml Template

```toml
# bacon configuration for Leptos projects
# Note: Use for linting only, NOT for cargo-leptos watch

default_job = "clippy-all"

[jobs.clippy-all]
command = ["cargo", "all-features", "clippy", "--no-deps", "--color", "always", "--", "-D", "warnings"]
need_stdout = false

[jobs.check-ssr]
command = ["cargo", "check", "--features", "ssr", "--color", "always"]
need_stdout = false

[jobs.check-hydrate]
command = ["cargo", "check", "--target", "wasm32-unknown-unknown", "--features", "hydrate", "--color", "always"]
need_stdout = false

[jobs.test]
command = ["cargo", "test", "--features", "ssr", "--color", "always"]
need_stdout = true

[keybindings]
a = "job:clippy-all"
s = "job:check-ssr"
h = "job:check-hydrate"
t = "job:test"
```

---

## SSR Mode Recommendations

| Route | Mode | Reason |
|-------|------|--------|
| `/compare/{id}` | OutOfOrder | Fast interactivity, stream photo data |
| `/rankings` | InOrder | Reliable page structure |
| `/about` | Static | No dynamic data |

---

## Utility Libraries Assessment

| Library | Use? | Purpose |
|---------|------|---------|
| leptos-use | YES | Intersection observer, localStorage, debounce |
| leptos-query | MAYBE | Matchup fetching (caching, SSR) |
| leptos-image | NO | Incompatible with S3 presigned URLs |
| thaw | REFERENCE | Learn patterns, don't depend |
| tailwind-fuse | MAYBE | Class composition for conditional styling |

---

## Files Explored

### Track A: Core Framework
- `/Users/ryzhakar/pp/_leptos-exploration/leptos/integrations/axum/src/lib.rs` - State injection
- `/Users/ryzhakar/pp/_leptos-exploration/cargo-leptos/README.md` - Build config
- `/Users/ryzhakar/pp/_leptos-exploration/leptos-use/src/use_intersection_observer.rs` - Lazy loading
- `/Users/ryzhakar/pp/_leptos-exploration/thaw/thaw/src/button/` - Component patterns
- `/Users/ryzhakar/pp/_leptos-exploration/start-axum/` - Project template

### Track B: Compiler-Driven Workflow
- `/Users/ryzhakar/pp/_leptos-exploration/leptos/.github/workflows/` - CI patterns
- `/Users/ryzhakar/pp/_leptos-exploration/cargo-leptos/src/command/` - CLI analysis
- `https://github.com/bram209/leptosfmt` - Formatter docs
- `https://github.com/axodotdev/cargo-dist/book/` - cargo-dist config
- `https://github.com/release-plz/release-plz/website/docs/` - release-plz config

### Track B: Investigation Reports
- `/Users/ryzhakar/pp/_leptos-exploration/REPORT-dual-target-linting.md`
- `/Users/ryzhakar/pp/_leptos-exploration/REPORT-cargo-leptos-cli.md`
- `/Users/ryzhakar/pp/_leptos-exploration/REPORT-leptosfmt-precommit.md`
- `/Users/ryzhakar/pp/_leptos-exploration/REPORT-ci-patterns.md`
- `/Users/ryzhakar/pp/_leptos-exploration/REPORT-cargo-dist-leptos.md`
- `/Users/ryzhakar/pp/_leptos-exploration/REPORT-release-plz-workspace.md`

---

## Next Steps

### Phase 0: Tooling Setup (Track B)
1. Install `cargo-all-features` from custom fork
2. Install `leptosfmt` for view! macro formatting
3. Create `bacon.toml` for linting jobs
4. Update `.pre-commit-config.yaml` with leptosfmt hook
5. Create `release-plz.toml` with per-package config
6. Update `dist-workspace.toml` to exclude webapp

### Phase 1: Leptos Migration (Track A)
1. Create `filmorator-web` crate with leptos metadata
2. Set up Tailwind v4 (one CSS file, no JS config)
3. Implement `ProgressiveImage` component with lazy loading
4. Implement `RankState` enum for type-safe ranking
5. Wire up `AppState` with dual S3 clients via `leptos_routes_with_context`

### Phase 2: CI/CD Updates
1. Update `.github/workflows/ci.yml` with Leptos-aware jobs
2. Add leptosfmt-action for formatting check
3. Configure WASM target installation
4. Add Docker build job for webapp releases

---

## Tool Installation Commands

```bash
# cargo-all-features (custom fork for arbitrary command support)
cargo install --git https://github.com/sabify/cargo-all-features --branch arbitrary-command-support

# leptosfmt (view! macro formatting)
cargo install leptosfmt

# cargo-leptos (build orchestration)
cargo install cargo-leptos --locked

# wasm-bindgen-cli (WASM tooling)
cargo install wasm-bindgen-cli

# Add WASM target
rustup target add wasm32-unknown-unknown
```
