# Legacy PoC Reference

This directory contains the original Proof of Concept implementation. It is **reference material**, not living code.

## Purpose

The PoC was built to learn. It succeeded at that purpose and is now superseded by the Vision architecture.

## What the PoC Is

A working webapp that:
- Syncs photos from S3 bucket listing on startup
- Creates per-session matchups at random
- Computes per-session Bradley-Terry rankings
- Serves string HTML templates

## What the PoC Is Not

The PoC is not:
- The target architecture (Vision is)
- A migration source (Vision is a different product)
- Code to be ported (patterns may be adapted, structure should not)

---

## Learnings Extracted

### Proven: Works

| Pattern | Location | Carry Forward |
|---------|----------|---------------|
| AWS SDK presigned URLs | `s3.rs` | Adapt dual-client pattern into typed `FromRef` state |
| AWS SDK bucket operations | `s3.rs` | List, get, presign operations |
| Bradley-Terry MM algorithm | `../filmorator-core/src/ranking.rs` | **Kept in place** - algorithm is architecture-agnostic |
| sqlx migrations | `../migrations/` | **Kept in place** - add new migrations, don't modify |
| sqlx query patterns | `db.rs` | Adapt for new schema |
| Axum routing with tower layers | `main.rs` | Leptos wraps Axum; patterns transfer |
| Docker compose local dev | `../docker-compose.yml` | **Kept in place** - extend for cargo-leptos |
| Config from environment | `config.rs` | Pattern reusable |

### Proven: Does Not Work

| Pattern | Location | Why It Failed | Vision Alternative |
|---------|----------|---------------|-------------------|
| Per-session rankings | `db.rs`, schema | Defeats crowdsourcing; each user isolated | Global per-campaign |
| Photos from bucket listing | `main.rs:72-79` | Requires restart, no campaign isolation | Manifest-driven |
| Matchups generated at runtime | `handlers/api.rs` | No coverage guarantee, no transitivity | snic-seeded pool in manifest |
| String HTML templates | `handlers/compare.rs`, `handlers/style.rs` | Type-unsafe, refactoring blind spot | Leptos components |
| Session = ranking scope | Entire architecture | Conceptual error | Campaign = ranking scope |

### Proven: Needs Refinement

| Pattern | PoC State | Evolution |
|---------|-----------|-----------|
| S3 client | Single struct, internal dual logic | Two typed clients via `FromRef` |
| Error handling | `thiserror` enum | Keep pattern, expand variants |
| State management | `AppState` with Arc internals | `FromRef` derive for Leptos context |

---

## Reference Usage Guide

### When Building AWS Integration

Reference `s3.rs` for:
- `aws_sdk_s3::Client` construction with custom endpoint
- Presigned URL generation with public URL override
- Bucket listing with prefix filtering

Do NOT replicate:
- Single client with internal branching
- Sync-on-startup pattern

### When Building Database Layer

Reference `db.rs` for:
- sqlx query macros
- UUID handling
- Timestamp handling

Do NOT replicate:
- Session-scoped queries
- Photo table structure (Vision uses manifest, not DB for photo metadata)

### When Building Error Handling

Reference `error.rs` for:
- `thiserror` derive pattern
- Axum `IntoResponse` implementation

Adapt for:
- Leptos `ServerFnError` integration
- Campaign-scoped errors

### When Building Config

Reference `config.rs` for:
- Environment variable loading pattern
- Optional vs required config

This pattern is largely reusable as-is.

---

## What Lives Outside Legacy

These components remain in the active codebase:

| Path | Reason |
|------|--------|
| `filmorator-core/` | Algorithm and types are architecture-agnostic |
| `migrations/` | Migration approach is correct; new migrations will be added |
| `docker-compose.yml` | Infrastructure is correct |
| `Cargo.toml` (workspace) | Workspace structure is correct |
| `.github/workflows/` | CI patterns will evolve |
| `.pre-commit-config.yaml` | Linting approach will evolve |
| `justfile` | Commands will evolve |
| `VISION.md` | The specification |
| `CLAUDE.md` | Project instructions |
| `.claude/` | Research artifacts, skills |

---

## Files in This Directory

```
_legacy/
└── filmorator-web/     # The PoC webapp
    ├── Cargo.toml      # Dependencies (reference for versions)
    └── src/
        ├── main.rs     # Entry point, routing, startup
        ├── config.rs   # Env config loading (reusable pattern)
        ├── db.rs       # sqlx patterns (adapt for new schema)
        ├── s3.rs       # AWS SDK patterns (adapt for typed clients)
        ├── state.rs    # AppState (replace with FromRef)
        ├── error.rs    # Error types (adapt)
        └── handlers/   # String templates (DO NOT REPLICATE)
            ├── mod.rs
            ├── api.rs
            ├── compare.rs
            ├── session.rs
            └── style.rs
```

---

## Anti-Patterns to Avoid

These patterns exist in the PoC and should NOT be carried forward:

1. **`sync_photos_from_s3` on startup** - Webapp should read manifest, not discover photos
2. **`create_matchup` generating random indices** - Matchups come from manifest pool
3. **`photo_ratings` table with `session_id` PK** - Ratings are per-campaign, not per-session
4. **String interpolation in handlers** - Use Leptos `view!` macro
5. **`impl IntoResponse for Template`** - No templates; Leptos handles rendering

---

## The Key Insight

The PoC and the Vision are **different products**:

| | PoC | Vision |
|-|-----|--------|
| Question answered | "Can we build a photo ranker?" | "Can we crowdsource rankings?" |
| User model | Single user, multiple sessions | Multiple users, one campaign |
| Data flow | Webapp discovers → serves → stores | CLI prepares → Webapp serves → stores |
| Ranking scope | Isolated per session | Aggregated per campaign |

Attempting "parity" between them is a category error. The Vision is not a migration target; it's a different system informed by PoC learnings.
