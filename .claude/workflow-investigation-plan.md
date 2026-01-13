# Compiler-Driven Workflow Investigation Plan

**Created**: 2026-01-13
**Status**: Planning (pre-indexed)
**Pre-Index**: `.claude/workflow-pre-index.md`
**Constitutional Constraints**: Correct By Construction, Simple Made Easy

## Pre-Index Discoveries (Already Known)

| Question | Answer | Source |
|----------|--------|--------|
| leptosfmt CI integration? | **leptosfmt-action** exists | awesome-leptos |
| Feature flag handling? | SSR/hydrate mutually exclusive | leptos lib.rs |
| WASM target needed? | `rustup target add wasm32-unknown-unknown` | leptos docs |
| leptosfmt installation? | `cargo install leptosfmt` | awesome-leptos |

## Objective

Determine how to maintain the current compiler-driven development workflow (clippy pedantic, bacon, pre-commit hooks, CI) when migrating to Leptos, which introduces:

1. **Dual compilation targets**: SSR (native) + hydrate (WASM)
2. **Feature-gated code**: `#[cfg(feature = "ssr")]` vs `#[cfg(feature = "hydrate")]`
3. **New build tool**: cargo-leptos (wraps cargo)
4. **New formatter**: leptosfmt (for `view!` macro)

## Current Workflow (Baseline)

```
justfile (commands) → pre-commit (local) → CI (GitHub Actions)
     ↓                      ↓                    ↓
  cargo fmt              doublify/pre-commit-rust  cargo fmt --check
  cargo clippy           clippy --fix              clippy --all-targets
  cargo test             cargo-check               cargo test
```

**Key files:**
- `.pre-commit-config.yaml` - fmt, cargo-check, clippy (pedantic, autofix)
- `.github/workflows/ci.yml` - fmt, clippy, build, test (SQLX_OFFLINE)
- `justfile` - dev, clippy, test, check, ci-check

## Investigation Questions

### Category A: Dual-Target Clippy/Tests

| Question | Why It Matters |
|----------|----------------|
| How to run clippy for both `ssr` and `hydrate` features? | Code is feature-gated; single-feature clippy misses bugs |
| Does `--all-features` work or cause conflicts? | SSR and hydrate may be mutually exclusive |
| How to test WASM-only code? | `wasm-bindgen-test` vs regular tests |
| Does clippy work with `wasm32-unknown-unknown` target? | Need to lint client code |

### Category B: cargo-leptos Integration

| Question | Why It Matters |
|----------|----------------|
| Can bacon wrap `cargo leptos watch`? | Bacon provides better diagnostics UX |
| Does cargo-leptos support `--message-format=json`? | Bacon requires this for parsing |
| What's the right `bacon.toml` configuration? | Jobs for ssr-check, wasm-check, watch |
| Does cargo-leptos auto-run clippy? | Avoid redundant checks |

### Category C: leptosfmt Integration

| Question | Why It Matters |
|----------|----------------|
| How to run leptosfmt in pre-commit? | `view!` macro needs special formatting |
| Does leptosfmt conflict with rustfmt? | Need both to coexist |
| Is there a pre-commit hook for leptosfmt? | Or need custom script |
| CI integration pattern? | Check formatting in GitHub Actions |

### Category D: Pre-commit Hooks

| Question | Why It Matters |
|----------|----------------|
| Does `doublify/pre-commit-rust` work with workspaces? | Current repo uses it |
| How to handle feature-gated clippy in hooks? | Need both ssr and hydrate checks |
| Performance impact of dual-target checks? | Pre-commit should be fast |
| Alternative: typos, cargo-deny in hooks? | Enhanced checks |

### Category E: CI Pipeline

| Question | Why It Matters |
|----------|----------------|
| How to install wasm32 target in CI? | `rustup target add wasm32-unknown-unknown` |
| cargo-leptos build in CI? | `cargo leptos build --release` |
| Caching strategy for WASM builds? | Reduce CI time |
| Artifact output (server binary + WASM)? | Deployment artifacts |

## Investigation Sources

### Primary (Cloned Repos)

| Repo | Location | Investigate |
|------|----------|-------------|
| start-axum | `~/pp/_leptos-exploration/start-axum/` | CI workflow if present |
| start-axum-workspace | `~/pp/_leptos-exploration/start-axum-workspace/` | Workspace CI patterns |
| cargo-leptos | `~/pp/_leptos-exploration/cargo-leptos/` | CLI args, JSON output, bacon compat |
| leptos | `~/pp/_leptos-exploration/leptos/` | Main repo CI, clippy config |

### Secondary (Web)

| Resource | URL | Why |
|----------|-----|-----|
| leptosfmt repo | https://github.com/bram209/leptosfmt | Installation, pre-commit hook |
| bacon docs | https://dystroy.org/bacon/ | cargo-leptos integration |
| Leptos CI examples | Search in leptos examples | Production CI patterns |

### Tertiary (Reference Projects)

| Project | Why |
|---------|-----|
| gallery-rs (`~/pp/gallery-rs`) | Existing CLI/webapp split, CI patterns |
| snic-rs (`~/pp/snic-rs`) | Workspace CI patterns |

## Parallel Agent Tasks (Refined)

Pre-index context: `.claude/workflow-pre-index.md`

### Agent B1: Dual-Target Linting Patterns
**Scope**: How leptos ecosystem actually runs clippy on dual targets
**Sources**:
- `~/pp/_leptos-exploration/leptos/.github/workflows/` (PRIMARY)
- `~/pp/_leptos-exploration/start-axum/`
**Output**: `~/pp/_leptos-exploration/REPORT-dual-target-linting.md`

**Known**: SSR/hydrate features are mutually exclusive
**Investigate**:
- [ ] Extract exact clippy commands from leptos CI
- [ ] How are WASM-specific lints configured?
- [ ] Are there clippy.toml configurations?
- [ ] Test patterns: wasm-bindgen-test vs cargo test?
- [ ] Does leptos use `cargo hack` for feature matrix?

### Agent B2: cargo-leptos CLI Analysis
**Scope**: Whether cargo-leptos can integrate with bacon
**Sources**:
- `~/pp/_leptos-exploration/cargo-leptos/src/command/` (PRIMARY)
- `~/pp/_leptos-exploration/cargo-leptos/README.md`
**Output**: `~/pp/_leptos-exploration/REPORT-cargo-leptos-cli.md`

**Known**: cargo-leptos coordinates server+client rebuilds
**Investigate**:
- [ ] Search for `message-format` in cargo-leptos source
- [ ] What CLI args does `cargo leptos watch` accept?
- [ ] Does it shell out to cargo with passthrough args?
- [ ] Can we run `cargo leptos build --message-format=json`?
- [ ] Alternative workflow: bacon for lint, cargo-leptos for serve only

### Agent B3: leptosfmt Pre-commit Integration
**Scope**: Local dev formatting (pre-commit, not just CI)
**Sources**:
- https://github.com/bram209/leptosfmt (web fetch README)
- Existing pre-commit patterns
**Output**: `~/pp/_leptos-exploration/REPORT-leptosfmt-precommit.md`

**Known**: leptosfmt exists, leptosfmt-action exists for CI
**Investigate**:
- [ ] Does leptosfmt provide a pre-commit hook ID?
- [ ] How to run: `leptosfmt --check` vs `leptosfmt --write`?
- [ ] Order: run rustfmt first, then leptosfmt?
- [ ] Does leptosfmt modify .rs files in-place?
- [ ] Standalone binary available? (for faster pre-commit)

### Agent B4: CI Pipeline Extraction
**Scope**: Extract working CI patterns from production repos
**Sources**:
- `~/pp/_leptos-exploration/leptos/.github/workflows/` (PRIMARY)
- `~/pp/_leptos-exploration/cargo-leptos/.github/workflows/`
**Output**: `~/pp/_leptos-exploration/REPORT-ci-patterns.md`

**Known**: Need WASM target, leptosfmt-action exists
**Investigate**:
- [ ] Extract complete job definitions for clippy/test
- [ ] Caching strategy for target/ with WASM builds
- [ ] How is cargo-leptos used in CI (if at all)?
- [ ] Artifact handling for server binary + WASM
- [ ] Any use of matrix strategy for features?

### Agent B5: cargo-dist + Leptos Compatibility (CRITICAL)
**Scope**: Can cargo-dist work with Leptos dual-artifact builds?
**Sources** (raw GitHub, not web):
- `https://raw.githubusercontent.com/axodotdev/cargo-dist/main/book/src/custom-builds.md`
- `https://raw.githubusercontent.com/axodotdev/cargo-dist/main/book/src/reference/config.md`
- `https://raw.githubusercontent.com/axodotdev/cargo-dist/main/book/src/workspaces/simple.md`
- `~/pp/_leptos-exploration/cargo-leptos/` (build output structure)
- Search GitHub for "leptos cargo-dist" examples
**Output**: `~/pp/_leptos-exploration/REPORT-cargo-dist-leptos.md`

**Known** (from pre-index):
- cargo-dist `build-command` is for NON-Cargo builds
- cargo-leptos IS a Cargo wrapper
- `extra-artifacts` may work for WASM
- `include` can add directories to archives

**Investigate**:
- [ ] Does `extra-artifacts` glob support `target/site/**/*`?
- [ ] Can cargo-leptos be invoked via build script wrapper?
- [ ] Does cargo-leptos respect `CARGO_DIST_TARGET` for cross-compilation?
- [ ] What do actual Leptos projects use for releases?
- [ ] Should CLI (cargo-dist) and webapp (Docker?) be separate?
- [ ] Can release-plz version both crates together?

**Constitutional Note**: If cargo-dist cannot handle Leptos, this is inherent complexity (dual-target) not incidental. Solution must not complect the release process.

### Agent B6: release-plz Workspace Configuration
**Scope**: How to configure release-plz for workspace with CLI + webapp
**Sources** (raw GitHub):
- `https://raw.githubusercontent.com/release-plz/release-plz/main/website/docs/config.md`
- `https://raw.githubusercontent.com/release-plz/release-plz/main/website/docs/github/quickstart.md`
- Current `.github/workflows/ci.yml` for reference
**Output**: `~/pp/_leptos-exploration/REPORT-release-plz-workspace.md`

**Known**:
- release-plz handles versioning and changelogs
- Supports `[[package]]` overrides per package
- `version_group` can sync versions across packages
- `publish = false` can disable crates.io publish per package

**Investigate**:
- [ ] How to configure: CLI publishes to crates.io, webapp doesn't?
- [ ] `version_group` for CLI + core, separate webapp version?
- [ ] Changelog per package or unified?
- [ ] Integration with cargo-dist tags?

## Expected Outputs

After investigation, update synthesis with:

1. **bacon.toml template** for Leptos development
2. **Updated .pre-commit-config.yaml** with leptosfmt + dual-target clippy
3. **Updated justfile** with Leptos-aware commands
4. **Updated CI workflow** for WASM + native builds

## Constitutional Compliance Checklist

Before recommending any pattern, verify:

### Simple Made Easy
- [ ] Does this add tools/complexity that could be avoided?
- [ ] Is there a simpler alternative that achieves the same goal?
- [ ] Are we entangling concerns that should be separate?

### Correct By Construction
- [ ] Does CI catch feature-gated bugs that local dev might miss?
- [ ] Are there compile-time checks we should add?
- [ ] Does the workflow leverage Rust's type system fully?

## Notes

- Current pre-commit uses `clippy --fix --allow-staged` - may conflict with workspace feature flags
- SQLX_OFFLINE already in use - good pattern, keep
- release-plz for releases - keep
