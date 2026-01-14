# Filmorator Implementation Plan

Stakeholder-testable increments. Each phase delivers a working system.

---

## Guiding Principles

### Gall's Law
> "A complex system that worked evolved from a simple system that worked."

Each increment produces a simple system that works. Later increments extend without breaking.

### First-Principles Risk Ordering

Riskiest assumptions tested first:

| Rank | Assumption | If Wrong |
|------|------------|----------|
| 1 | 3-photo ranking UX is intuitive | Product fails |
| 2 | Presigned URLs work end-to-end | Architecture fails |
| 3 | Bradley-Terry aggregation works | Rankings meaningless |
| 4 | Sessions persist across visits | Contributions lost |
| 5 | CLI workflow is usable | Owner experience fails |

Each increment de-risks one assumption.

### Increment Structure

Every increment has:
- **Stakeholder Test**: What can be tested
- **Technical Scope**: What gets built
- **Type Safety**: Compile-time guarantees added
- **Deferred**: What's explicitly NOT built
- **Success Criteria**: How we know it works

---

## Pre-Increment: Existing Assets

| Asset | Location | Status |
|-------|----------|--------|
| Bradley-Terry algorithm | `filmorator-core/src/ranking.rs` | ✅ Ready |
| Core models | `filmorator-core/src/models.rs` | ⚠️ Needs campaign_id |
| Matchup generation | `filmorator-core/src/matchup.rs` | ⚠️ Uses random, not snic |
| Infrastructure | `docker-compose.yml` | ✅ PostgreSQL + MinIO ready |
| Legacy patterns | `_legacy/filmorator-web/` | 📚 Reference only |
| Leptos research | `.claude/synthesis-leptos-migration.md` | ✅ Decisions made |

---

## Increment 0: Foundation

**Stakeholder Test**: "I can open localhost:3000 and see the Filmorator landing page"

### Technical Scope

```
filmorator-web/
├── Cargo.toml           # Leptos metadata, features
├── src/
│   ├── main.rs          # Axum server entry (ssr)
│   ├── lib.rs           # WASM entry (hydrate)
│   └── app.rs           # Root component + router shell
└── style/
    └── tailwind.css     # @import "tailwindcss";
```

- Leptos project with cargo-leptos
- Tailwind v4 (standalone binary, no JS)
- Single route: `/` → "Welcome to Filmorator"
- Docker-compose service for webapp

### Developer Infrastructure

- `bacon.toml` for linting jobs
- `.pre-commit-config.yaml` with leptosfmt
- Updated `justfile` with Leptos commands
- CI workflow for dual-target clippy

### Type Safety

None yet (pure scaffold).

### Deferred

- All routes except `/`
- Database, S3, business logic

### Success Criteria

```bash
cargo leptos serve
# → localhost:3000 shows styled landing page
# → Hot reload works
# → `just lint` passes
```

---

## Increment 1: Ranking UX

**Stakeholder Test**: "I can click 3 photos in order and see my selection"

### Technical Scope

```
filmorator-web/src/
├── components/
│   ├── mod.rs
│   ├── photo_card.rs    # Photo with rank indicator
│   └── ranking_selector.rs  # 3-photo comparison
└── pages/
    ├── mod.rs
    └── compare.rs       # Comparison page (static photos)
```

- `RankState` enum (Empty → One → Two → Complete)
- `PhotoCard` component with gold/silver/bronze borders
- `RankingSelector` orchestrating 3 cards
- Route: `/{campaign_id}/compare` (campaign_id ignored, static photos)
- Placeholder images (local or URLs)

### Type Safety

```rust
pub enum RankState {
    Empty,
    One(usize),
    Two(usize, usize),
    Complete(usize, usize, usize),
}
```

Invalid ranking sequences are unrepresentable. Cannot select 4th photo. Cannot duplicate selection.

### Deferred

- Real photos from S3
- Persistence
- Campaign logic

### Success Criteria

1. Open `/test/compare`
2. Click photo A → gold border, "1st" badge
3. Click photo B → silver border, "2nd" badge
4. Click photo C → bronze border, "3rd" badge
5. Click photo B again → removes B and C from selection
6. Complete selection → console logs `[A, B, C]`

**First-Principles Validation**: Stakeholder judges if clicking order is intuitive before any backend work.

---

## Increment 2: S3 Integration

**Stakeholder Test**: "I see MY photos, not placeholders"

### Technical Scope

```
filmorator-web/src/
├── server/
│   ├── mod.rs
│   ├── manifest.rs      # Load manifest from S3
│   └── images.rs        # Presigned URL generation
└── state.rs             # AppState with dual S3 clients
```

- `AppState` with `FromRef` derive
- `S3InternalClient` (minio:9000) for server operations
- `S3PresignClient` (localhost:9000) for browser URLs
- `leptos_routes_with_context` injection
- Server function: `get_manifest(campaign_id)`
- Server function: `get_image_url(campaign_id, tier, idx)`
- Route: `/{campaign_id}/img/{tier}/{idx}` → presigned redirect

### Type Safety

```rust
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub s3_internal: S3InternalClient,  // Newtype
    pub s3_presign: S3PresignClient,    // Newtype
}
```

Distinct types prevent using wrong client.

### Deferred

- Database
- Sessions
- Multiple campaigns (hardcode test campaign)

### Success Criteria

1. Upload `manifest.json` + photos to MinIO manually
2. Open `/{campaign_id}/compare`
3. See real photos from S3

**First-Principles Validation**: Docker networking and presigned URLs work before adding database complexity.

---

## Increment 3: Persistence

**Stakeholder Test**: "My rankings are saved and affect the result"

### Technical Scope

```sql
-- migrations/YYYYMMDD_campaigns.sql
CREATE TABLE campaigns (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    owner_secret TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE comparisons (
    id UUID PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    matchup_indices INT[] NOT NULL,
    ranked_indices INT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE campaign_ratings (
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    photo_idx INT NOT NULL,
    strength FLOAT8 NOT NULL,
    uncertainty FLOAT8 NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (campaign_id, photo_idx)
);
```

```
filmorator-web/src/
├── server/
│   ├── compare.rs       # Submit comparison
│   └── ranking.rs       # Get global ranking
└── pages/
    └── ranking.rs       # Rankings display page
```

- `submit_comparison` server function
- `get_ranking` server function
- Ranking page: `/{campaign_id}/ranking`
- Rating recomputation on comparison submit

### Type Safety

```rust
pub enum CampaignStatus { Active, Closed }
```

Status is enum, not string.

### Deferred

- Sessions (all comparisons anonymous)
- CLI (manually insert campaign row)
- Owner management

### Success Criteria

1. Insert campaign row in DB manually
2. Submit 5+ comparisons
3. View `/test/ranking` → see Bradley-Terry ranked photos

**First-Principles Validation**: Aggregation produces sensible rankings before session complexity.

---

## Increment 4: Sessions

**Stakeholder Test**: "My contributions are tracked separately from others"

### Technical Scope

```sql
-- migrations/YYYYMMDD_sessions.sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id),
    name TEXT,
    comparison_count INT DEFAULT 0,
    excluded BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_active_at TIMESTAMPTZ DEFAULT NOW()
);

ALTER TABLE comparisons ADD COLUMN session_id UUID REFERENCES sessions(id);
```

```
filmorator-web/src/
├── server/
│   └── session.rs       # Session management
└── components/
    ├── contribution_count.rs
    └── name_input.rs
```

- Cookie-based session tracking
- `get_or_create_session` server function
- Contribution count display
- Optional name field
- Session ID in comparisons

### Type Safety

```rust
pub struct SessionId(Uuid);  // Newtype
```

### Deferred

- CLI
- Owner management
- Contributor list

### Success Criteria

1. Open `/test/compare` in Browser A → submit 3 comparisons
2. Open `/test/compare` in Browser B → different session, count starts at 0
3. Enter name in Browser A → refresh → name persists

**First-Principles Validation**: Sessions persist across browser visits.

---

## Increment 5: CLI

**Stakeholder Test**: "I can create a new campaign from my photos"

### Technical Scope

```
filmorator-cli/
├── Cargo.toml
└── src/
    ├── main.rs          # clap CLI entry
    ├── commands/
    │   ├── mod.rs
    │   ├── create.rs    # campaign create
    │   ├── add_images.rs # campaign add-images
    │   └── generate.rs  # campaign generate-matchups
    └── image.rs         # Resize to 3 tiers
```

- `filmorator campaign create "My Roll"`
- `filmorator campaign add-images ./photos/`
- `filmorator campaign generate-matchups`
- Owner secret generation and display
- snic integration for matchup pool

### Type Safety

```rust
pub struct OwnerSecret(String);  // Distinct from CampaignId
pub struct CampaignId(String);
```

### Deferred

- Owner management UI
- Feedback mechanisms

### Success Criteria

```bash
filmorator campaign create "Test Roll"
# → Campaign created: abc123
# → Management URL: http://localhost:3000/abc123/manage/secret456
# → Share URL: http://localhost:3000/abc123

filmorator campaign add-images ./my-photos/
# → Added 50 photos

filmorator campaign generate-matchups
# → Generated 47 matchups
```

**First-Principles Validation**: CLI workflow is usable before building management UI.

---

## Increment 6: Owner Management

**Stakeholder Test**: "I can close my campaign and see who contributed"

### Technical Scope

```
filmorator-web/src/
├── pages/
│   └── manage.rs        # Owner dashboard
└── server/
    └── manage.rs        # Close/reopen, exclude
```

- Route: `/{campaign_id}/manage/{secret}`
- Contributor list with counts
- Close campaign action
- Reopen campaign action
- Exclude session action

### Type Safety

Routes validate secret matches campaign.

### Deferred

- Feedback UI
- Impact display

### Success Criteria

1. CLI creates campaign with secret
2. Open management URL → see contributor list
3. Close campaign → compare page shows "Campaign closed"
4. Reopen → compare page works again
5. Exclude a session → ranking recalculates without their data

---

## Increment 7: Feedback UI

**Stakeholder Test**: "I see my impact and how my ranking differs from the crowd"

### Technical Scope

```
filmorator-web/src/
├── components/
│   ├── progress_bar.rs
│   ├── impact_feedback.rs
│   └── contributor_list.rs
└── pages/
    └── personal.rs      # Personal vs aggregate
```

- Campaign progress bar
- Impact feedback on submission ("Photo 7 moved up 2 positions")
- Personal ranking derivation (on-demand Bradley-Terry on session's comparisons)
- Unlock mechanics (threshold-based)
- `/{campaign_id}/me` route

### Type Safety

```rust
pub enum UnlockState {
    Locked { comparisons_needed: u32 },
    Unlocked,
}
```

### Deferred

- Polish

### Success Criteria

1. Submit comparison → see "Photo X moved up N positions"
2. View progress bar → shows "45% complete"
3. View personal ranking → differs from aggregate
4. New user → "Complete 5 more comparisons to unlock rankings"

---

## Increment 8: Polish

**Stakeholder Test**: "I can share this with friends and it works on their phones"

### Technical Scope

- Mobile-responsive Tailwind styling
- Progressive image loading (thumb → preview)
- Lazy loading via `use_intersection_observer`
- Rate limiting middleware
- Abuse detection flags
- Error handling UI
- Lightbox component for full-size photos

### Success Criteria

1. Works on mobile without horizontal scroll
2. Images load progressively (placeholder → thumb → preview)
3. Rapid submissions are rate-limited
4. Network errors show user-friendly message

---

## Dependency Graph

```
Increment 0: Foundation
    ↓
Increment 1: Ranking UX
    ↓
Increment 2: S3 Integration
    ↓
Increment 3: Persistence
    ↓
Increment 4: Sessions
    ↓ ↘
    ↓   Increment 5: CLI (can parallelize)
    ↓ ↙
Increment 6: Owner Management
    ↓
Increment 7: Feedback UI
    ↓
Increment 8: Polish
```

CLI (Increment 5) can be built in parallel with Session work (Increment 4) since they share only the database schema.

---

## Parallelization Opportunities

### After Increment 0 (Foundation)

| Track A (Frontend) | Track B (Backend) | Track C (CLI) |
|-------------------|-------------------|---------------|
| Components | Server functions | — |
| RankState | S3 integration | — |
| PhotoCard | Manifest loading | — |

### After Increment 3 (Persistence)

| Track A (Webapp) | Track C (CLI) |
|-----------------|---------------|
| Session UI | create command |
| Contribution count | add-images command |
| Name input | generate-matchups |

---

## Files Modified Per Increment

### Increment 0
- NEW: `filmorator-web/` (entire crate)
- NEW: `bacon.toml`
- MOD: `.pre-commit-config.yaml`
- MOD: `justfile`
- MOD: `docker-compose.yml`
- MOD: `Cargo.toml` (workspace)
- MOD: `.github/workflows/ci.yml`

### Increment 1
- NEW: `filmorator-web/src/components/*.rs`
- NEW: `filmorator-web/src/pages/compare.rs`
- MOD: `filmorator-web/src/app.rs` (routing)

### Increment 2
- NEW: `filmorator-web/src/server/*.rs`
- NEW: `filmorator-web/src/state.rs`
- MOD: `filmorator-web/src/main.rs` (state injection)

### Increment 3
- NEW: `migrations/YYYYMMDD_*.sql`
- MOD: `filmorator-web/src/server/*.rs`
- NEW: `filmorator-web/src/pages/ranking.rs`
- MOD: `docker-compose.yml` (ensure migrations run)

### Increment 4
- NEW: `migrations/YYYYMMDD_sessions.sql`
- NEW: `filmorator-web/src/server/session.rs`
- NEW: `filmorator-web/src/components/contribution_count.rs`
- NEW: `filmorator-web/src/components/name_input.rs`

### Increment 5
- NEW: `filmorator-cli/` (entire crate)
- MOD: `Cargo.toml` (workspace)
- MOD: `dist-workspace.toml` (add CLI)
- MOD: `filmorator-core/src/matchup.rs` (snic integration)

### Increment 6
- NEW: `filmorator-web/src/pages/manage.rs`
- NEW: `filmorator-web/src/server/manage.rs`

### Increment 7
- NEW: `filmorator-web/src/components/progress_bar.rs`
- NEW: `filmorator-web/src/components/impact_feedback.rs`
- NEW: `filmorator-web/src/pages/personal.rs`
- MOD: `filmorator-web/src/pages/compare.rs` (feedback display)

### Increment 8
- MOD: All component styling
- NEW: `filmorator-web/src/components/progressive_image.rs`
- NEW: `filmorator-web/src/components/lightbox.rs`
- MOD: Server middleware (rate limiting)

---

## Testing Strategy Per Increment

| Increment | Test Focus |
|-----------|------------|
| 0 | Cargo builds, cargo-leptos serves, CI passes |
| 1 | RankState unit tests, component renders |
| 2 | S3 integration tests (with testcontainers or mock) |
| 3 | Database tests, Bradley-Terry aggregation |
| 4 | Session creation, cookie persistence |
| 5 | CLI unit tests, image processing |
| 6 | Management actions, authorization |
| 7 | Impact calculation, unlock logic |
| 8 | E2E tests, mobile viewport |

---

## Constitutional Compliance

### First-Principles (T0)
Each increment tests one risky assumption before building dependent features.

### Decomplect (T1)
Each increment is independently testable. No increment requires future work to be usable.

### Correct By Construction (T2)
Each increment adds type safety for its domain:
- `RankState` prevents invalid selection sequences
- Typed S3 clients prevent endpoint confusion
- `CampaignStatus` enum prevents invalid states
- Newtypes distinguish `SessionId`, `CampaignId`, `OwnerSecret`

### Code Speaks (T3)
Increment names describe stakeholder capability, not technical work.

---

## References

| Document | Purpose |
|----------|---------|
| `spec/ARCHITECTURE_PLAN.md` | Technical architecture (routes, schema) |
| `spec/product-spec.md` | Product decisions |
| `spec/user-stories.md` | Requirements with AC |
| `.claude/synthesis-leptos-migration.md` | Leptos tooling decisions |
| `_legacy/README.md` | PoC patterns (reference) |
