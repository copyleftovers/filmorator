# Filmorator Vision

Crowdsource photo rankings. Multiple anonymous participants contribute pairwise comparisons to derive a single global ranking via Bradley-Terry model.

## Core Concept

**The problem:** You have N photos and want to rank them by quality/preference. Doing this alone is biased. Asking others to rank all N is tedious.

**The solution:** Show 3 photos at a time. Ask: "Rank these best to worst." Aggregate many such comparisons from different people into one global ranking using Bradley-Terry maximum likelihood estimation.

**Why it works:** Each 3-way comparison yields 3 pairwise relationships. With enough comparisons covering all pairs transitively (via snic GBER decomposition), we get a statistically valid global ranking.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                            CLI                                   │
│  filmorator campaign create "Summer 2024"                       │
│  filmorator campaign add-images ./photos/                       │
│  filmorator campaign generate-matchups                          │
│  filmorator campaign status                                     │
└─────────────────────┬───────────────────────────────────────────┘
                      │ writes
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                            S3                                    │
│  campaigns/{id}/manifest.json    ← campaign metadata + matchups │
│  campaigns/{id}/thumb/{file}     ← 400px thumbnails             │
│  campaigns/{id}/preview/{file}   ← 2048px previews              │
│  campaigns/{id}/original/{file}  ← full resolution              │
└─────────────────────┬───────────────────────────────────────────┘
                      │ reads
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                          Webapp                                  │
│  GET  /{campaign_id}/compare     ← serve matchup from pool      │
│  POST /{campaign_id}/compare     ← record comparison to DB      │
│  GET  /{campaign_id}/progress    ← show contribution stats      │
│  GET  /{campaign_id}/ranking     ← show current global ranking  │
└─────────────────────┬───────────────────────────────────────────┘
                      │ writes comparisons
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                         PostgreSQL                               │
│  sessions           ← anonymous participant tracking            │
│  comparisons        ← raw comparison data (session + ranking)   │
│  campaign_ratings   ← global Bradley-Terry strengths (computed) │
└─────────────────────────────────────────────────────────────────┘
```

### Key Principle: CLI Writes, Webapp Reads (from S3)

Following the gallery-rs pattern:

| Component | Responsibilities |
|-----------|-----------------|
| **CLI** | Create campaigns, process images (resize to 3 tiers), upload to S3, generate snic seed matchups, update manifests |
| **Webapp** | Serve matchups from pre-generated pool, collect comparisons, compute/display global rankings, never modify S3 |
| **S3** | Single source of truth for campaign content and matchup pools |
| **PostgreSQL** | Comparison results, session tracking, computed global ratings |

**No server restarts.** CLI updates S3 manifest → webapp sees changes immediately.

## Frontend: Leptos

The webapp uses Leptos—a full-stack Rust framework with compile-time guarantees for UI code.

### Why Leptos

This project follows Correctness by Construction. String-templated HTML/CSS/JS bypasses Rust's compile-time guarantees—a blind spot in an otherwise type-safe system. Leptos closes this gap:

| Aspect | String HTML | Leptos |
|--------|-------------|--------|
| Type safety | None | Props, routes, state verified at compile time |
| Refactoring | Find-replace, hope | Compiler catches breaks |
| Reactivity | Manual DOM/JS | Signals, derived state |
| Components | Copy-paste | Composable, typed, testable |

### Component Architecture

```
filmorator-web/src/
├── app.rs                 # Router, root layout
├── components/
│   ├── photo_card.rs      # Single photo with rank indicator
│   ├── ranking_selector.rs # 3-photo comparison with undo
│   ├── lightbox.rs        # Full-screen zoom view
│   ├── progress_bar.rs    # Campaign completion
│   └── image.rs           # Progressive loading (thumb→preview→original)
├── pages/
│   ├── campaign.rs        # Landing page for campaign
│   ├── compare.rs         # Main comparison UI
│   └── ranking.rs         # Global ranking display
└── server/
    ├── matchup.rs         # Server function: get next matchup
    └── compare.rs         # Server function: submit comparison
```

### Key Patterns

**Reactive ranking (enables undo):**
```rust
let ranking = create_rw_signal::<Vec<usize>>(vec![]);

// Click adds to ranking
on:click=move |_| ranking.update(|r| r.push(idx))

// Click again removes (undo)
on:click=move |_| ranking.update(|r| r.retain(|&x| x != idx))
```

**Progressive image loading:**
```rust
#[component]
fn ProgressiveImage(thumb: String, preview: String) -> impl IntoView {
    let src = create_rw_signal(thumb.clone());
    view! {
        <img
            src=move || src.get()
            on:load=move |_| src.set(preview.clone())
        />
    }
}
```

**Server functions (type-safe RPC):**
```rust
#[server(GetMatchup)]
async fn get_matchup(campaign_id: String) -> Result<Matchup, ServerFnError> {
    // Runs on server, called from client like a function
}
```

### Build

Uses `cargo-leptos` for coordinated server + WASM builds:

```bash
cargo leptos watch  # Dev with hot reload
cargo leptos build --release  # Production
```

## Campaigns

A campaign is a ranking project for a specific set of images.

```json
{
  "id": "a1b2c3d4",
  "name": "Summer 2024 Film Selects",
  "description": "Best shots from Portra 400 rolls",
  "created_at": "2024-01-15T10:00:00Z",
  "photos": [
    {"filename": "001.jpg", "hash": "sha256:...", "width": 3000, "height": 2000},
    ...
  ],
  "matchup_pool": [
    [0, 5, 12],
    [3, 7, 19],
    ...
  ],
  "completion_target": 3,
  "status": "active"
}
```

### Campaign Lifecycle

1. **Create**: `filmorator campaign create "Name"`
2. **Add images**: `filmorator campaign add-images ./folder/` (processes to 3 tiers, uploads)
3. **Generate matchups**: `filmorator campaign generate-matchups` (snic GBER seed)
4. **Activate**: Campaign goes live at `/{id}/compare`
5. **Collect**: Participants submit comparisons, global ranking updates
6. **Complete**: Reach completion threshold, export final ranking

### Multiple Simultaneous Campaigns

Different campaigns run independently:
- `/abc123/compare` — Campaign A
- `/def456/compare` — Campaign B
- Each has own matchup pool, ratings, completion status

## Matchup Generation (snic)

Use snic_core's GBER decomposition to generate O(n log n) matchups that guarantee:
- Every photo appears in at least one matchup
- Transitive coverage: any two photos can be compared via chain of shared matchups
- Efficient coverage with minimal redundancy

```rust
use snic_core::network::LocalMatchupsManager;

let manager = LocalMatchupsManager::new(num_photos, matchup_size);
let seed_matchups: Vec<Vec<u32>> = manager.generate_all();
```

### Matchup Serving

Pre-generated pool stored in manifest. Webapp serves from pool:

1. Participant requests matchup
2. Webapp picks unserved matchup from pool (round-robin or random)
3. Participant submits ranking
4. Comparison stored in DB
5. Repeat until pool exhausted or completion threshold reached

## Completion Threshold

**Question:** When does a campaign have "enough" data?

**Proposal:** Multiple of snic coverage.

| Multiplier | Meaning |
|------------|---------|
| 1× snic | Each matchup compared once (minimum coverage) |
| 2× snic | Each matchup compared twice on average |
| 3× snic | High confidence (recommended default) |

```
completion_target = 3  // in manifest
progress = total_comparisons / (matchup_pool.length * completion_target)
```

At 1× snic, we have transitive coverage. Additional rounds increase confidence by:
- Reducing variance from individual preferences
- Detecting inconsistent/spam responses
- Strengthening Bradley-Terry estimates

## Rankings

### Per-Campaign Global Ranking

All comparisons within a campaign aggregate to ONE ranking:

```sql
-- Aggregate all pairwise wins across all sessions
SELECT
  winner_idx,
  loser_idx,
  COUNT(*) as win_count
FROM comparisons
WHERE campaign_id = $1
GROUP BY winner_idx, loser_idx
```

Bradley-Terry MLE on aggregated wins → global strength parameters → ranking.

### Sessions ≠ Rankings

Sessions exist for:
- **Abuse detection**: Flag sessions with inconsistent patterns
- **Analytics**: Track participation, completion rates
- **Future features**: User accounts, contribution history

Sessions do NOT produce separate rankings. One campaign = one global ranking.

## Image Presentation

Photos are the stars. Presentation requirements:

### Aspect Ratio

**NO CROPS. EVER.**

Original aspect ratios preserved at all times. UI adapts to images, not vice versa.

### Progressive Loading

1. **Thumbnails** (400px): Initial grid, fast load
2. **Previews** (2048px): Comparison view, good detail
3. **Originals** (full): Lightbox/zoom, full appreciation

### Lightbox

Click to expand. Zoom to inspect. These are photographs meant to be appreciated.

### Processing Pipeline (CLI)

```
Input: original.jpg (any size)
  ↓
Validate: JPEG only (film scans are JPEG)
  ↓
Resize (Lanczos3, preserve aspect):
  - thumb:    400px max dimension, 85% quality
  - preview:  2048px max dimension, 90% quality
  - original: unchanged, 100% quality
  ↓
Upload to S3: campaigns/{id}/{tier}/{filename}
```

## User Experience

### Mobile-First

Primary use case: quick comparisons on phone. Design for:
- Touch-friendly tap targets
- Vertical scrolling
- Swipe gestures (future)
- Works offline-ish (progressive web app potential)

### Ranking Interaction

Current: Click photos in order (1st, 2nd, 3rd). Gold/silver/bronze borders.

**Needed improvements:**
- Undo individual selections (not just "clear all")
- Drag to reorder
- Keyboard shortcuts (1, 2, 3)

### Gratitude & Accomplishment

Open design question: How to reward participants?

Ideas to explore:
- Show contribution count: "You've ranked 47 matchups!"
- Show campaign progress: "Campaign is 68% complete, thanks to you"
- Reveal position after contributing: "Your comparison moved photo X from #12 to #8"
- Unlock final ranking after N contributions
- Shareable "I helped rank this" badge

## Data Model

### Campaign Manifest (S3)

```json
{
  "id": "string",
  "name": "string",
  "description": "string",
  "created_at": "ISO8601",
  "photos": [
    {
      "filename": "string",
      "hash": "sha256:string",
      "width": "number",
      "height": "number"
    }
  ],
  "matchup_pool": [[0,1,2], [3,4,5], ...],
  "matchup_size": 3,
  "completion_target": 3,
  "status": "draft|active|completed"
}
```

### PostgreSQL Schema

```sql
-- Sessions (anonymous participants)
CREATE TABLE sessions (
  id UUID PRIMARY KEY,
  campaign_id TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  last_active_at TIMESTAMPTZ DEFAULT NOW(),
  comparison_count INT DEFAULT 0
);

-- Raw comparisons
CREATE TABLE comparisons (
  id UUID PRIMARY KEY,
  campaign_id TEXT NOT NULL,
  session_id UUID REFERENCES sessions(id),
  matchup_indices INT[] NOT NULL,        -- [0, 5, 12]
  ranked_indices INT[] NOT NULL,         -- [5, 0, 12] (winner first)
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Computed global ratings (refreshed periodically or on-demand)
CREATE TABLE campaign_ratings (
  campaign_id TEXT NOT NULL,
  photo_idx INT NOT NULL,
  strength FLOAT8 NOT NULL,
  uncertainty FLOAT8 NOT NULL,
  comparison_count INT NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  PRIMARY KEY (campaign_id, photo_idx)
);

-- Indexes
CREATE INDEX idx_comparisons_campaign ON comparisons(campaign_id);
CREATE INDEX idx_sessions_campaign ON sessions(campaign_id);
```

## CLI Commands

```bash
# Campaign management
filmorator campaign list
filmorator campaign create "Name" --description "Description"
filmorator campaign delete <id>

# Image management
filmorator campaign add-images <id> ./path/to/images/
filmorator campaign remove-image <id> <filename>
filmorator campaign list-images <id>

# Matchup generation
filmorator campaign generate-matchups <id> [--size 3] [--algorithm snic]

# Status & control
filmorator campaign status <id>
filmorator campaign activate <id>
filmorator campaign complete <id>

# Export
filmorator campaign export-ranking <id> --format json|csv
filmorator campaign export-data <id>  # Full comparison dataset
```

## API Routes

```
# Campaign participation
GET  /{campaign_id}              → Campaign landing/info page
GET  /{campaign_id}/compare      → Comparison UI
POST /{campaign_id}/compare      → Submit comparison
GET  /{campaign_id}/progress     → Participant's contribution stats
GET  /{campaign_id}/ranking      → Current global ranking (if visible)

# Images (presigned URL redirects)
GET  /{campaign_id}/img/{tier}/{idx}

# Health
GET  /health
```

## Implementation Phases

### Phase 0: Leptos Migration
- [ ] Set up cargo-leptos workspace
- [ ] Create Leptos app shell with router
- [ ] Port compare page to Leptos components
- [ ] Implement reactive ranking selector (with undo)
- [ ] Remove string HTML templates

### Phase 1: Campaign Foundation
- [ ] CLI skeleton with campaign CRUD
- [ ] S3 manifest read/write
- [ ] Image processing pipeline (3 tiers)
- [ ] Campaign-aware routing in webapp

### Phase 2: snic Integration
- [ ] Integrate snic_core for matchup generation
- [ ] Store matchup pool in manifest
- [ ] Serve matchups from pool via server functions

### Phase 3: Global Rankings
- [ ] Comparison storage in PostgreSQL
- [ ] Bradley-Terry aggregation across sessions
- [ ] Global ranking display component

### Phase 4: UX Polish
- [ ] Mobile-responsive Tailwind styling
- [ ] Lightbox component with zoom
- [ ] Progressive image loading component
- [ ] Progress/gratitude features

### Phase 5: Production Hardening
- [ ] Rate limiting
- [ ] Abuse detection
- [ ] Caching strategy
- [ ] Monitoring/alerting

## Reference

- **Leptos**: Full-stack Rust framework ([leptos.dev](https://leptos.dev))
- **gallery-rs**: CLI/webapp split pattern (`~/pp/gallery-rs`)
- **snic-rs**: Matchup generation (`~/pp/snic-rs`)
- **Bradley-Terry**: [Wikipedia](https://en.wikipedia.org/wiki/Bradley%E2%80%93Terry_model)
