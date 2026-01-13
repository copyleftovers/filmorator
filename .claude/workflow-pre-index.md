# Workflow Investigation Pre-Index

**Created**: 2026-01-13
**Purpose**: Starting context for Track B agent delegation

---

## Non-Negotiable Tooling

| Tool | Purpose | Current Config |
|------|---------|----------------|
| **cargo-dist** | Binary distribution | `dist-workspace.toml` |
| **release-plz** | Versioning, release PRs | `.github/workflows/ci.yml` |
| **pre-commit** | Local lint/format | `.pre-commit-config.yaml` |
| **justfile** | Command runner | `justfile` |

These MUST continue working after Leptos migration.

---

## ⚠️ CRITICAL TENSION: cargo-dist + Leptos

### The Problem

**cargo-dist** builds native binaries for distribution:
- Targets: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, etc.
- Produces: tarballs, installers, GitHub Releases

**Leptos** produces TWO artifacts:
- Server binary (native) ✅ cargo-dist can handle
- WASM bundle (wasm32-unknown-unknown) ❌ cargo-dist has NO WASM support

### Current Config (`dist-workspace.toml`)

```toml
[workspace]
members = ["cargo:filmorator-web"]

[dist]
targets = ["aarch64-apple-darwin", "aarch64-unknown-linux-gnu",
           "x86_64-apple-darwin", "x86_64-unknown-linux-gnu",
           "x86_64-pc-windows-msvc"]
installers = []
```

### Potential Solutions (From cargo-dist docs)

**GOOD NEWS**: cargo-dist has features that MAY enable Leptos support:

#### 1. `build-command` (Experimental)
> "A command to run in your package's root directory to build its binaries"

```toml
[dist]
build-command = ["cargo", "leptos", "build", "--release"]
```

This could invoke cargo-leptos instead of cargo build!

#### 2. `extra-artifacts`
> "Specifies extra artifacts to build and upload. Users can download these directly."

```toml
[[extra-artifacts]]
build = ["cargo", "leptos", "build", "--release"]
artifacts = ["target/site/**/*"]
```

Could capture WASM bundle as additional artifact.

#### 3. `include`
> "Manually specifies additional files or directories to add to archives/installers."

```toml
[dist]
include = ["target/site/"]
```

### cargo-dist Custom Builds (From Docs)

**Key Finding**: `build-command` is for NON-CARGO builds:
> "When releasing software in languages other than Rust or JavaScript, you'll need to tell dist how to build it."

**Environment**: Provides `CARGO_DIST_TARGET` for cross-compilation.

**Implication**: cargo-leptos IS a Cargo wrapper, so `build-command` may not be the right approach. Need to investigate:
1. Can cargo-dist call cargo-leptos as cargo?
2. Should we use `extra-artifacts` instead?
3. Do we need a custom build script wrapper?

### Investigation Required

- [ ] Does `build-command` work when cargo-leptos wraps cargo?
- [ ] Can `extra-artifacts` glob pattern capture `target/site/`?
- [ ] Does cargo-leptos respect `CARGO_DIST_TARGET`?
- [ ] What does the leptos ecosystem actually use for releases?
- [ ] Should CLI and webapp be in separate cargo-dist configs?

---

## release-plz Integration

### What It Does
- Conventional commits → automatic version bumps
- Creates release PRs on each commit
- Publishes to crates.io on merge

### Current Workflow (`.github/workflows/ci.yml`)
```yaml
- name: Create or update release PR
  uses: release-plz/action@v0.5
  with:
    command: release-pr
```

### Leptos Compatibility
- ✅ Versioning works (it's cargo-based)
- ✅ Changelog generation works
- ⚠️ May need adjustment if webapp isn't published to crates.io

---

## Key Discoveries

### 1. leptosfmt-action EXISTS

**URL**: https://github.com/LesnyRumcajs/leptosfmt-action
**Purpose**: GitHub Action for leptosfmt CI integration

This solves the CI formatting question directly. No custom scripting needed.

### 2. Feature Flags Are Mutually Exclusive

From leptos lib.rs documentation:

| Flag | Purpose | Build Target |
|------|---------|--------------|
| `csr` | Client-side rendering | Browser (WASM) |
| `ssr` | Server-side rendering | Server (native) |
| `hydrate` | Add interactivity to SSR | Browser (WASM) |

**Implication**: Cannot use `--all-features` for clippy. Must run separately:
- `cargo clippy --features ssr` (server code)
- `cargo clippy --target wasm32-unknown-unknown --features hydrate` (client code)

### 3. Required Tooling

```bash
# Essential
cargo install cargo-leptos --locked
rustup target add wasm32-unknown-unknown

# Formatting
cargo install leptosfmt  # or use standalone binary
```

### 4. Project Entry Points

| File | Purpose | Feature |
|------|---------|---------|
| `src/main.rs` | Server entry point | `ssr` |
| `src/lib.rs` | Client entry point (WASM) | `hydrate` |
| `src/app.rs` | Shared application component | Both |

---

## awesome-leptos Tooling Extract

### Development Tools

| Tool | URL | Purpose |
|------|-----|---------|
| cargo-leptos | https://github.com/leptos-rs/cargo-leptos | Coordinated server+client rebuilds |
| leptosfmt | https://github.com/bram209/leptosfmt | `view!` macro formatting |
| leptosfmt-action | https://github.com/LesnyRumcajs/leptosfmt-action | CI integration |
| leptos-fmt (VSCode) | https://github.com/codeitlikemiley/leptos-fmt | Editor integration |
| cargo-runner (VSCode) | https://github.com/codeitlikemiley/cargo-runner | Contextual cargo commands |

### NOT Found in awesome-leptos

- bacon integration patterns
- pre-commit hook configurations
- clippy configuration for dual-target
- testing patterns

**Implication**: These must be investigated from source repos directly.

---

## Leptos MCP Documentation Sections

Available via `mcp__plugin_leptos-mcp_leptos__get-documentation`:

| Section | Relevant For |
|---------|--------------|
| `getting-started` | Project structure, installation |
| `server-functions` | SSR patterns, extractors |
| `components` | Component architecture |
| `signals` | Reactivity patterns |
| `resources` | Async data loading |
| `actions` | Mutations, forms |
| `routing` | Navigation |
| `error-handling` | Error patterns |
| `suspense` | Loading states |

---

## Dual-Target Clippy Strategy (Hypothesis)

Based on feature flag mutual exclusivity, proposed CI pattern:

```yaml
jobs:
  lint-server:
    steps:
      - run: cargo clippy --features ssr -- -D warnings

  lint-client:
    steps:
      - run: rustup target add wasm32-unknown-unknown
      - run: cargo clippy --target wasm32-unknown-unknown --features hydrate -- -D warnings
```

**To verify**: Check leptos main repo CI for actual pattern used.

---

## Pre-commit Hypothesis

Current pre-commit uses `doublify/pre-commit-rust` which runs:
- `cargo fmt`
- `cargo check`
- `cargo clippy`

**Problem**: Single invocation won't cover both features.

**Options to investigate**:
1. Custom hook script running both feature combinations
2. Separate hooks for ssr/hydrate
3. Only lint shared code in pre-commit, full lint in CI

---

## bacon.toml Hypothesis

bacon uses cargo's `--message-format=json` for error parsing.

**Question**: Does `cargo leptos watch` support this?

**Fallback**: Use bacon only for clippy/check, cargo-leptos for serve:

```toml
# bacon.toml
[jobs.check-ssr]
command = ["cargo", "check", "--features", "ssr"]

[jobs.check-hydrate]
command = ["cargo", "check", "--target", "wasm32-unknown-unknown", "--features", "hydrate"]

[jobs.clippy-ssr]
command = ["cargo", "clippy", "--features", "ssr"]
```

---

## Files to Investigate

### In Cloned Repos

| Path | Look For |
|------|----------|
| `leptos/.github/workflows/ci.yml` | Dual-target clippy pattern |
| `cargo-leptos/src/command/` | CLI argument handling |
| `start-axum/.github/` | Template CI if present |

### Raw GitHub Documentation (Agents Should Crawl)

**cargo-dist** (https://github.com/axodotdev/cargo-dist):
| File | Purpose |
|------|---------|
| `book/src/custom-builds.md` | `build-command` for cargo-leptos |
| `book/src/reference/config.md` | Full config reference |
| `book/src/workspaces/simple.md` | Workspace examples |
| `book/src/ci/customizing.md` | CI customization |

**release-plz** (https://github.com/release-plz/release-plz):
| File | Purpose |
|------|---------|
| `website/docs/config.md` | Configuration reference |
| `website/docs/github/quickstart.md` | GitHub Actions setup |

**leptosfmt** (https://github.com/bram209/leptosfmt):
| File | Purpose |
|------|---------|
| `README.md` | CLI usage, pre-commit |

**bacon** (https://github.com/Canop/bacon):
| File | Purpose |
|------|---------|
| `website/docs/config.md` | bacon.toml configuration |

---

## Constitutional Notes

### Simple Made Easy

**Risk**: Adding 4 tools (cargo-leptos, leptosfmt, leptosfmt-action, dual-target clippy) is complexity.

**Mitigation**: Each tool serves a distinct, non-overlapping purpose:
- cargo-leptos: build orchestration (inherent)
- leptosfmt: `view!` formatting (inherent - rustfmt can't do this)
- leptosfmt-action: CI (reuse existing, don't custom script)
- dual-target clippy: correctness (inherent - features are mutually exclusive)

### Correct By Construction

**Goal**: CI must catch bugs that local dev misses.

Local dev typically runs one feature at a time. CI must ensure:
- Both targets compile
- Both targets pass clippy
- Formatting is consistent
