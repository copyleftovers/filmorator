Upon starting, check this index of manifestos first: 'https://raw.githubusercontent.com/ryzhakar/LLM_MANIFESTOS/refs/heads/main/README.md'.
Use the 'manifest-oath' skill to swear to any of those manifestos when prompted.
This project being a rust project and striving to be idiomatic, use the 'correctness-by-construction' manifesto by default.
If continuing a session after context compaction - reswear to the active manifestos anew.
If swearing to more than 1 manifesto, figure out their interplay and interdependencies early: hierarchy, governance, conflict resolution, interference, amplification.
Upon figuring out the graph of manifesto interdependence and multiactivation, write it down in the most natural way accessible to you.

Delegate often and well.
Generally, you would want to use simpler models for any subagents, unless there's a good reason to do otherwise.
For any given delegation, you need to make an explicit decision whether to retain the conversation or now.
Rely on externalized context for delegation as a first-class citizen, prefering it to the handing-down the conversation history whenever possible.
Context, instructions and preferences are externalized as manifestos, plans, artifacts, operational notes, etc.

Plans must survive handoff to agents who lack your context. Use defensive-planning skill to do so.

If anything can be delegated and done in parallell, use multiple parallell agents.
One of the workflows where this pattern lends itself beautifully is objective fault analysis based on each of the active manifestos by separate agents.

## Project State

**Phase**: Post-PoC, pre-implementation. The PoC taught lessons, Vision was extracted, reimplementation pending.

**Codebase layout**:
- `filmorator-core/` — Active. Algorithm and types (architecture-agnostic).
- `_legacy/` — Reference only. PoC webapp with patterns to adapt, not replicate.
- `filmorator-web/` — Does not exist yet. Will be Leptos webapp.
- `filmorator-cli/` — Does not exist yet. Will create campaigns.

**Key documents**:
- `VISION.md` — The specification. This is what we're building.
- `_legacy/README.md` — What the PoC proved works/doesn't work.
- `.claude/synthesis-leptos-migration.md` — Technical decisions for Leptos migration.

## Strategic Context

Read `VISION.md` before any architectural work. The Vision describes a **different product** from the PoC:

| Aspect | PoC (legacy) | Vision (target) |
|--------|--------------|-----------------|
| Rankings | Per-session | Global per-campaign |
| Photo source | Bucket listing | S3 manifest |
| Matchups | Random at runtime | snic-seeded pool |
| Entry point | Webapp syncs on start | CLI prepares, webapp serves |

Reference architecture: `~/pp/gallery-rs` (CLI writes S3, webapp reads).

## Operational Constraints

- **Dual S3 client remains necessary**: `internal_client` (minio:9000) vs `presign_client` (localhost:9000). In Leptos, use typed clients via `FromRef`.
- **Axum 0.7**: Routes use `:param`, not `{param}`. Leptos wraps Axum.
- **Campaign-scoped routes**: `/{campaign_id}/compare`, not `/compare`.
- **Manifest-driven**: Photos and matchup pools come from S3 manifest JSON, not database or bucket listing.
- **Leptos documentation**: Use the Leptos MCP server tools (`mcp__plugin_leptos-mcp_leptos__*`).

## Legacy Reference

The `_legacy/` directory contains the PoC webapp. Use it as reference for:
- AWS SDK patterns (`s3.rs`)
- sqlx query patterns (`db.rs`)
- Config loading (`config.rs`)

Do NOT replicate:
- String HTML templates (`handlers/`)
- Per-session architecture
- Sync-on-startup pattern

See `_legacy/README.md` for detailed guidance.

## Debugging

Project-specific skill: `.claude/skills/filmorator-debug/`

Note: Debug skill references PoC routes and patterns. Will need updating after Leptos implementation.

Chrome MCP network/console tracking is lazy-start—call the tool BEFORE the action you want to capture.
