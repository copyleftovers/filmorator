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

## Strategic Context

Read `VISION.md` before architectural work. Current implementation is MVP scaffolding—don't optimize toward current structure.

Key gaps: per-session rankings (should be global), no campaigns, no CLI, server restarts required.

Reference architecture: `~/pp/gallery-rs` (CLI writes S3, webapp reads).

## Operational Constraints

- **Dual S3 client is load-bearing**: `internal_client` (minio:9000) vs `presign_client` (localhost:9000). Don't consolidate.
- **Axum 0.7**: Routes use `:param`, not `{param}`
- **API uses position (int), not id (UUID)**: `/img/thumb/42` is photo at position 42
- **No image processing**: Upload identical files to `original/`, `preview/`, `thumb/` externally
- **Don't add string HTML templates**: Target is Leptos. Current templates are technical debt, not a pattern to extend.

## Debugging

Project-specific skill: `.claude/skills/filmorator-debug/`

Chrome MCP network/console tracking is lazy-start—call the tool BEFORE the action you want to capture.
