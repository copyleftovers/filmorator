# Filmorator Architecture Plan

Technical architecture for implementing `product-spec.md`. This document describes HOW; product-spec describes WHAT.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                            CLI                                   │
│  filmorator campaign create "Name"                              │
│  filmorator campaign add-images ./photos/                       │
│  filmorator campaign generate-matchups                          │
└─────────────────────┬───────────────────────────────────────────┘
                      │ writes (immutable after creation)
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                            S3                                    │
│  campaigns/{id}/manifest.json    ← photos + matchup pool ONLY   │
│  campaigns/{id}/thumb/{file}     ← 400px thumbnails             │
│  campaigns/{id}/preview/{file}   ← 2048px previews              │
│  campaigns/{id}/original/{file}  ← full resolution              │
└─────────────────────────────────────────────────────────────────┘
                      │ reads (never writes)
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                          Webapp                                  │
│                                                                  │
│  Participant Routes:                                             │
│    GET  /{campaign_id}              → Landing page               │
│    GET  /{campaign_id}/compare      → Comparison UI              │
│    POST /{campaign_id}/compare      → Submit comparison          │
│    GET  /{campaign_id}/progress     → Stats + unlock status      │
│    GET  /{campaign_id}/ranking      → Global ranking (if unlocked)│
│    GET  /{campaign_id}/me           → Personal vs aggregate      │
│                                                                  │
│  Owner Routes (capability-based, no session):                    │
│    GET  /{campaign_id}/manage/{secret}     → Management dashboard│
│    POST /{campaign_id}/manage/{secret}/close   → Close campaign  │
│    POST /{campaign_id}/manage/{secret}/reopen  → Reopen campaign │
│    POST /{campaign_id}/manage/{secret}/exclude/{session_id}      │
│                                                                  │
│  Image Routes:                                                   │
│    GET  /{campaign_id}/img/{tier}/{idx}    → Presigned redirect  │
└─────────────────────┬───────────────────────────────────────────┘
                      │ reads + writes
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                         PostgreSQL                               │
│                                                                  │
│  campaigns         ← status, secrets, metadata (mutable)        │
│  sessions          ← participant tracking (name optional)       │
│  comparisons       ← raw comparison data                        │
│  campaign_ratings  ← global Bradley-Terry (computed cache)      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Key Architectural Decisions

### Decision: S3 Manifest is Immutable

**Rationale**: Product spec requires campaigns to be immutable after photo upload. S3 manifest contains photos and matchup pool only. Campaign status (active/closed) lives in PostgreSQL.

**Implication**: Webapp never writes to S3. CLI writes once at creation. Status changes go to DB.

### Decision: Campaign Status in Database

**Rationale**: Owners need to close/reopen campaigns without CLI access. Management URL provides capability. Status must be mutable by webapp.

**Schema**:
```sql
CREATE TABLE campaigns (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    owner_secret TEXT NOT NULL,  -- For management URL
    status TEXT NOT NULL DEFAULT 'active',  -- active | closed
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Decision: Fixed 3-Photo Matchups

**Rationale**: 2 is binary (boring), 4+ is cognitively taxing on mobile. 3 yields 3 pairwise comparisons per interaction. Hardcoded, not configurable.

### Decision: Sessionless Owner Actions

**Rationale**: Management URL IS the capability. No owner session needed. Owner can participate as a separate anonymous session. Simplest model, consistent with "no accounts."

### Decision: Personal Rankings Derived On-Demand

**Rationale**: To show "your rankings vs aggregate", query participant's comparisons and run Bradley-Terry on their subset. Stateless, always current. Acceptable performance for typical session sizes (10-50 comparisons).

---

## Component Responsibilities

### CLI (`filmorator-cli`)

| Command | Writes To | Behavior |
|---------|-----------|----------|
| `campaign create` | S3 (manifest), DB (campaigns row) | Creates empty manifest, generates owner secret |
| `campaign add-images` | S3 (images + manifest update) | Processes to 3 tiers, updates manifest photos array |
| `campaign generate-matchups` | S3 (manifest update) | Runs snic, writes matchup_pool to manifest |
| `campaign status` | — | Reads from S3 + DB, displays info |
| `campaign list` | — | Lists from DB |

**Note**: No `campaign delete` command. Per product spec, deletion is not supported.

### Webapp (`filmorator-web`)

| Route | Reads From | Writes To |
|-------|-----------|-----------|
| `/{id}/compare` GET | S3 (manifest), DB (session) | DB (session activity) |
| `/{id}/compare` POST | — | DB (comparisons, session count) |
| `/{id}/ranking` | DB (campaign_ratings) | — |
| `/{id}/me` | DB (session comparisons) | — |
| `/{id}/manage/{secret}/*` | DB (campaigns, sessions) | DB (campaigns status, sessions excluded) |

**Webapp never writes to S3.**

### S3 Structure

```
campaigns/
└── {campaign_id}/
    ├── manifest.json      # Immutable after matchup generation
    ├── thumb/
    │   ├── 001.jpg
    │   └── ...
    ├── preview/
    │   └── ...
    └── original/
        └── ...
```

**Manifest schema** (immutable):
```json
{
  "id": "string",
  "name": "string",
  "created_at": "ISO8601",
  "photos": [
    {"filename": "001.jpg", "width": 3000, "height": 2000}
  ],
  "matchup_pool": [[0, 1, 2], [3, 4, 5], ...],
  "matchup_size": 3,
  "completion_target": 3
}
```

**Note**: No `status` field. Status lives in DB.

---

## Database Schema

```sql
-- Campaign metadata (mutable status)
CREATE TABLE campaigns (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    owner_secret TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'closed')),
    threshold_reached_at TIMESTAMPTZ,  -- When statistical threshold met
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Participant sessions
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    name TEXT,  -- Optional, participant-provided
    excluded BOOLEAN DEFAULT FALSE,  -- Owner can exclude suspicious sessions
    comparison_count INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_active_at TIMESTAMPTZ DEFAULT NOW()
);

-- Raw comparisons
CREATE TABLE comparisons (
    id UUID PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    session_id UUID NOT NULL REFERENCES sessions(id),
    matchup_indices INT[] NOT NULL,   -- [0, 5, 12]
    ranked_indices INT[] NOT NULL,    -- [5, 0, 12] (best first)
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Computed global ratings (cache, refreshed on read)
CREATE TABLE campaign_ratings (
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    photo_idx INT NOT NULL,
    strength FLOAT8 NOT NULL,
    uncertainty FLOAT8 NOT NULL,
    comparison_count INT NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (campaign_id, photo_idx)
);

-- Indexes
CREATE INDEX idx_sessions_campaign ON sessions(campaign_id);
CREATE INDEX idx_comparisons_campaign ON comparisons(campaign_id);
CREATE INDEX idx_comparisons_session ON comparisons(session_id);
```

---

## Frontend: Leptos

### Why Leptos

Per Correct By Construction manifesto: String-templated HTML bypasses Rust's compile-time guarantees. Leptos provides type-safe components, verified routes, and reactive state.

### Component Structure

```
filmorator-web/src/
├── app.rs                    # Router, root layout
├── components/
│   ├── photo_card.rs         # Photo with rank indicator
│   ├── ranking_selector.rs   # 3-photo comparison (RankState enum)
│   ├── progress_bar.rs       # Campaign completion
│   ├── contributor_list.rs   # Social proof
│   └── image.rs              # Progressive loading
├── pages/
│   ├── landing.rs            # Campaign info
│   ├── compare.rs            # Main comparison UI
│   ├── ranking.rs            # Global ranking display
│   ├── personal.rs           # Personal vs aggregate
│   └── manage.rs             # Owner dashboard
└── server/
    ├── matchup.rs            # Get next matchup
    ├── compare.rs            # Submit comparison
    ├── ranking.rs            # Get rankings (global + personal)
    └── manage.rs             # Owner actions
```

### Key Type: RankState

Per Correct By Construction: Invalid states unrepresentable.

```rust
pub enum RankState {
    Empty,
    One(usize),
    Two(usize, usize),
    Complete(usize, usize, usize),
}

impl RankState {
    pub fn toggle(self, idx: usize) -> Self {
        match self {
            RankState::Empty => RankState::One(idx),
            RankState::One(a) if a == idx => RankState::Empty,
            RankState::One(a) => RankState::Two(a, idx),
            RankState::Two(a, b) if b == idx => RankState::One(a),
            RankState::Two(a, b) => RankState::Complete(a, b, idx),
            RankState::Complete(a, b, c) if c == idx => RankState::Two(a, b),
            RankState::Complete(a, b, c) => RankState::Complete(a, b, c), // No change if re-clicking earlier
        }
    }
}
```

---

## Algorithm: Bradley-Terry Ranking

### Global Ranking

All non-excluded comparisons within a campaign aggregate to ONE ranking:

```sql
SELECT winner_idx, loser_idx, COUNT(*) as win_count
FROM comparisons c
JOIN sessions s ON c.session_id = s.id
WHERE c.campaign_id = $1 AND NOT s.excluded
GROUP BY winner_idx, loser_idx
```

Run MM algorithm (see `filmorator-core/src/ranking.rs`) on aggregated wins.

### Personal Ranking

Derived on-demand from single session's comparisons:

```sql
SELECT winner_idx, loser_idx, COUNT(*) as win_count
FROM comparisons
WHERE session_id = $1
GROUP BY winner_idx, loser_idx
```

Run same MM algorithm on subset.

### Statistical Threshold

Campaign is "statistically meaningful" when:
1. Every photo appears in at least one compared matchup
2. Transitive coverage exists (any two photos comparable via chain)
3. Total comparisons ≥ `matchup_pool.len() * completion_target`

Default `completion_target = 3` means each matchup compared 3× on average.

---

## Matchup Generation: snic

Use `snic_core` GBER decomposition for O(n log n) coverage:

```rust
use snic_core::network::LocalMatchupsManager;

let manager = LocalMatchupsManager::new(num_photos, 3);
let seed_matchups: Vec<[u32; 3]> = manager.generate_all();
```

Guarantees:
- Every photo in at least one matchup
- Transitive coverage (compare any pair via chain)
- Minimal redundancy

---

## Image Pipeline

### CLI Processing

```
Input: original.jpg (any size, JPEG only)
  ↓
Validate: JPEG, ≤50MB, readable
  ↓
Resize (Lanczos3, preserve aspect):
  - thumb:    400px max dimension, 85% quality
  - preview:  2048px max dimension, 90% quality
  - original: unchanged
  ↓
Upload to S3: campaigns/{id}/{tier}/{filename}
```

### Webapp Serving

Presigned URL redirect pattern:
1. Client requests `/{campaign_id}/img/preview/5`
2. Server generates presigned URL for `campaigns/{id}/preview/005.jpg`
3. Redirect (302) to presigned URL
4. Client fetches directly from S3

---

## Dual S3 Client Architecture

**Constraint**: Docker networking requires different endpoints.

| Client | Endpoint | Purpose |
|--------|----------|---------|
| `internal_client` | `minio:9000` | Server-side operations (list, get manifest) |
| `presign_client` | `localhost:9000` | Generate browser-accessible presigned URLs |

Injected via Leptos `FromRef` pattern:

```rust
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub db_pool: PgPool,
    pub s3_internal: S3Client,
    pub s3_presign: S3Client,
}
```

---

## Implementation Phases

### Phase 0: Foundation
- [ ] Leptos workspace setup with cargo-leptos
- [ ] Database schema migration (campaigns table)
- [ ] AppState with dual S3 clients
- [ ] Basic routing structure

### Phase 1: Core Comparison Flow
- [ ] Load manifest from S3
- [ ] Serve matchup from pool
- [ ] Submit comparison to DB
- [ ] RankState component with undo

### Phase 2: Rankings & Feedback
- [ ] Global ranking computation + cache
- [ ] Personal ranking derivation
- [ ] Progress/contribution display
- [ ] Impact feedback ("Photo X moved up")

### Phase 3: Management
- [ ] Owner dashboard route
- [ ] Close/reopen actions
- [ ] Contributor list view
- [ ] Session exclusion

### Phase 4: CLI
- [ ] Campaign create (manifest + DB)
- [ ] Add images (processing + upload)
- [ ] Generate matchups (snic)

### Phase 5: Polish
- [ ] Mobile-responsive styling
- [ ] Progressive image loading
- [ ] Lightbox component
- [ ] Rate limiting

---

## References

| Resource | Purpose |
|----------|---------|
| `product-spec.md` | Product decisions (WHAT) |
| `personas.md` | User model |
| `user-stories.md` | Requirements with AC |
| `.claude/synthesis-leptos-migration.md` | Leptos technical details |
| `_legacy/README.md` | PoC learnings |
| `filmorator-core/src/ranking.rs` | Bradley-Terry implementation |
