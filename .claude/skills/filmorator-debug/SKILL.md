---
name: filmorator-debug
description: >-
  End-to-end debugging for the filmorator webapp (Rust/Axum + PostgreSQL + MinIO + Docker).
  Use when: images don't load, API returns errors, data not persisting, photos missing from
  storage, containers won't start, UI not updating, or any investigation across the stack.
  Provides symptom-to-tool mapping, layer-by-layer diagnostics, Chrome MCP browser testing,
  and common failure pattern resolution.
---

# Filmorator Debugging

## Architecture

```
Browser (localhost:3000) ─────► Axum App ─────► PostgreSQL (db:5432)
         │                         │
         │ presigned URL redirect  │ internal S3
         ▼                         ▼
    MinIO (localhost:9000)    MinIO (minio:9000)
```

**Critical config:** `S3_PUBLIC_URL` must be `localhost:9000` for browser-accessible presigned URLs. Internal operations use `minio:9000`.

**Image tiers:** `original/`, `preview/`, `thumb/` in bucket `filmorator`

## Symptom → Action

| Symptom | First Check | Reference |
|---------|-------------|-----------|
| Images don't load | `curl -v localhost:3000/img/thumb/0` | [browser.md](references/browser.md) |
| API errors | `curl localhost:3000/api/progress` | [api.md](references/api.md) |
| Data not persisting | `docker compose exec db psql ...` | [database.md](references/database.md) |
| Photos missing | `mc ls local/filmorator/original/` | [storage.md](references/storage.md) |
| Container won't start | `docker compose logs <service>` | [infrastructure.md](references/infrastructure.md) |
| UI not updating | Chrome screenshot + network | [browser.md](references/browser.md) |

## Quick Health Check

Run in parallel for rapid triage:

```bash
docker compose ps                    # Containers healthy?
curl -s localhost:3000/api/progress  # API responding?
mc ls local/filmorator/original/     # Images in S3?
docker compose logs app --tail=20    # Recent errors?
```

## Key Files

| File | When to Read |
|------|--------------|
| `filmorator-web/src/main.rs` | Routing issues, startup failures |
| `filmorator-web/src/s3.rs` | Image loading failures (dual-client architecture) |
| `filmorator-web/src/handlers/` | API behavior issues |
| `migrations/` | Schema questions |

## Route Reference

```
GET  /compare              → Compare UI
POST /api/matchup          → Get next 3 photos
POST /api/compare          → Submit ranking
GET  /api/progress         → Completion stats
GET  /api/ranking          → Session rankings
GET  /img/:tier/:id        → Presigned URL redirect (axum 0.7 syntax)
```

## Common Fixes

**Images 404:** Route syntax wrong. Axum 0.7 uses `:param`, not `{param}`.

**Images 403:** Presigned URL has wrong host. Check `S3_PUBLIC_URL` env var.

**No photos synced:** Check `mc ls local/filmorator/original/` and app startup logs.

**Session resets:** Progress is cookie-based. Curl needs `-c`/`-b` flags.

## Detailed References

- [browser.md](references/browser.md) - Chrome MCP tools, network debugging, visual verification
- [infrastructure.md](references/infrastructure.md) - Docker, container management, service health
- [storage.md](references/storage.md) - MinIO/S3, bucket operations, presigned URLs
- [database.md](references/database.md) - PostgreSQL queries, schema, data inspection
- [api.md](references/api.md) - Endpoint testing, request tracing, curl patterns
- [failure-patterns.md](references/failure-patterns.md) - Detailed diagnosis for common issues
