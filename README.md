# Filmorator

Crowdsource photo rankings via pairwise comparison. Bradley-Terry model aggregates preferences into global ranking.

**See [VISION.md](VISION.md) for the full architectural vision.**

## Current State

Working MVP with significant gaps from vision:

| Aspect | Vision | Current |
|--------|--------|---------|
| Frontend | Leptos (type-safe) | String HTML templates |
| Rankings | Global (crowdsourced) | Per-session |
| Content | Multi-campaign | Single image set |
| Updates | CLI-driven, no restarts | Server restart required |
| Matchups | snic GBER seeded | Random shuffle |
| Device | Mobile-first | Desktop-only |
| Images | Lightbox + zoom | Basic grid |

**This is scaffolding.** Core comparison flow works. Architecture needs rework.

## Run

```bash
docker compose up -d
```

Access: http://localhost:3000/compare

## Add Test Images

No image processing. Upload identical files to all 3 tiers:

```bash
mc alias set local http://localhost:9000 minioadmin minioadmin
mc cp image.jpg local/filmorator/original/
mc cp image.jpg local/filmorator/preview/
mc cp image.jpg local/filmorator/thumb/
docker compose restart app  # Required until CLI exists
```

## Architecture (Current)

```
Browser → Axum (3000) → PostgreSQL (5432)
   ↓ redirect              ↓ internal
MinIO (9000) ←─────────────┘
```

Presigned URLs: `S3_PUBLIC_URL` (browser) vs `AWS_ENDPOINT_URL` (Docker network).

## What Works

- Compare UI with 3-photo matchups
- Gold/silver/bronze ranking selection
- Session-based progress tracking
- S3 presigned URL image serving
- PostgreSQL persistence

## What's Missing

- [ ] Leptos migration (replace string HTML with typed components)
- [ ] CLI for campaign/image management
- [ ] Global ranking aggregation
- [ ] snic matchup generation
- [ ] Campaign concept (multi-campaign support)
- [ ] Mobile-responsive design
- [ ] Lightbox/zoom for images
- [ ] Ranking UX (undo, reorder)
- [ ] Completion thresholds
- [ ] User gratitude/progress features
