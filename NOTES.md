# Finish notes — 2026-08-29 (portfolio audit)

Decision needed. `_legacy/` holds a complete, previously-working ~1090-line end-to-end app
(axum routing, sqlx DB layer, S3 client, 8 routes, per-session Bradley-Terry ranking, own
Dockerfile). Not runnable today only because it was cut from the workspace `members` list and
needs Postgres + MinIO — no run docs, no tests of its own.

The current rewrite (branch `feature/increment-0-landing`) is a bare "Coming soon" Leptos
landing page with zero functionality, stalled ~7.5 months.

Recommendation: REVIVE `_legacy` — re-add to workspace `members`, add a compose file for
Postgres+MinIO and run docs, add a smoke test — and drop the Leptos rewrite. Exception: if
crowdsourced GLOBAL rankings (not per-session) are a hard requirement, that is a multi-week
greenfield, not a finish.

Context: sibling dir `printshop` was the abandoned Python precursor of this same app; `snic-rs`
is the shared comparison-network library and is already published to PyPI as `snic` v0.1.0.
The app idea has failed to land twice — revive, do not restart.

This branch is 2 commits ahead of `main`.
