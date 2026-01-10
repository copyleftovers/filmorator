# Violation Registry

## Discovery Context

Six parallel agents audited the codebase on 2025-01-10, each bound to one oath:

| Agent | Oath | Focus |
|-------|------|-------|
| CbC | Correct By Construction | Panics, silent failures, invalid states |
| SME | Simple Made Easy | Runtime complexity, complection |
| DEC | Decomplect | Braided concerns, function length |
| YAGNI | Build For Today | Dead code, unused fields |
| DRY | Knowledge Has One Home | Duplicated patterns |
| CS | Code Speaks | Comments, naming, type signatures |

**Governing oath:** CbC. All others interpreted through its lens.

---

## Violation Index

### V001: Panic on invalid PORT
```
file: filmorator-web/src/main.rs:28
code: .expect("PORT must be a number")
oath: CbC
fix:  Config::from_env() -> Result<Config, ConfigError>
```

### V002: Panic on missing DATABASE_URL
```
file: filmorator-web/src/main.rs:30
code: .expect("DATABASE_URL must be set")
oath: CbC
fix:  Config::from_env() -> Result
```

### V003: Panic on missing FILMORATOR_BUCKET
```
file: filmorator-web/src/main.rs:31
code: .expect("FILMORATOR_BUCKET must be set")
oath: CbC
fix:  Config::from_env() -> Result
```

### V004: Panic on presigning config
```
file: filmorator-web/src/s3.rs:77
code: .expect("valid presigning config")
oath: CbC
fix:  presign_url() -> anyhow::Result<String>
```

### V005: Silent failure on sync
```
file: filmorator-web/src/main.rs:48-50
code: if let Err(e) = sync_photos_from_s3(...) { tracing::warn!(...); }
oath: CbC
fix:  sync_photos_from_s3(...).await?
```

### V006: Silent failure on seed matchup creation
```
file: filmorator-web/src/handlers/api.rs:100-102
code: if let Err(e) = db::create_matchup(...) { tracing::error!(...); }
oath: CbC
fix:  db::create_matchup(...).await?
```

### V007: Silent failure on rating save
```
file: filmorator-web/src/handlers/api.rs:239-241
code: if let Err(e) = db::save_ratings(...) { tracing::error!(...); }
oath: CbC
fix:  db::save_ratings(...).await?
```

### V008: Silent failure on photo upsert
```
file: filmorator-web/src/handlers/api.rs:354-358
code: if let Err(e) = db::upsert_photo(...) { tracing::error!(...); }
oath: CbC
fix:  db::upsert_photo(...).await?
```

### V009: Error treated as false
```
file: filmorator-web/src/handlers/api.rs:93-95
code: .unwrap_or(false)
oath: CbC
fix:  db::has_seed_matchups(...).await?
```

### V010: Empty cookie fallback
```
file: filmorator-web/src/handlers/session.rs:51
code: .unwrap_or_else(|_| HeaderValue::from_static(""))
oath: CbC
fix:  session_cookie_header() -> Result<HeaderValue, AppError>
```

### V011: Overflow to zero (count_photos)
```
file: filmorator-web/src/db.rs:36
code: u32::try_from(count).unwrap_or(0)
oath: CbC, DRY
fix:  .map_err(|_| sqlx::Error::Protocol(...))?
```

### V012: Overflow to zero (save_ratings)
```
file: filmorator-web/src/db.rs:180
code: i32::try_from(rating.photo_idx).unwrap_or(0)
oath: CbC
fix:  .map_err(|_| sqlx::Error::Protocol(...))?
```

### V013: Overflow to zero (upsert_photo)
```
file: filmorator-web/src/db.rs:254
code: i32::try_from(position).unwrap_or(0)
oath: CbC
fix:  .map_err(|_| sqlx::Error::Protocol(...))?
```

### V014: Overflow to zero (get_photo_filename)
```
file: filmorator-web/src/db.rs:276
code: i32::try_from(position).unwrap_or(0)
oath: CbC
fix:  .map_err(|_| sqlx::Error::Protocol(...))?
```

### V015: Overflow to zero (sync loop)
```
file: filmorator-web/src/main.rs:79
code: u32::try_from(position).unwrap_or(0)
oath: CbC
fix:  .map_err(|_| anyhow::anyhow!(...))?
```

### V016: NULL array to empty vec
```
file: filmorator-web/src/db.rs:79-80
code: .unwrap_or_default()
oath: CbC, DRY
fix:  i32_vec_to_u32_vec(row.get(...))?
```

### V017: NULL array to empty vec (comparisons)
```
file: filmorator-web/src/db.rs:136-137
code: .unwrap_or_default()
oath: CbC, DRY
fix:  i32_vec_to_u32_vec(row.get(...))?
```

### V018: NULL array to empty vec (pending seed)
```
file: filmorator-web/src/db.rs:231-232
code: .unwrap_or_default()
oath: CbC, DRY
fix:  matchup_from_row(row)?
```

### V019: Circular fallback constant
```
file: filmorator-web/src/handlers/api.rs:75
code: u32::try_from(MATCHUP_SIZE).unwrap_or(3)
oath: CbC, SME
fix:  const MATCHUP_SIZE: u32 = 3
```

### V020: Circular fallback constant (duplicate)
```
file: filmorator-web/src/handlers/api.rs:144
code: u32::try_from(MATCHUP_SIZE).unwrap_or(3)
oath: CbC, SME
fix:  const MATCHUP_SIZE: u32 = 3
```

### V021: Unused struct SessionRanking
```
file: filmorator-core/src/models.rs:128-135
code: pub struct SessionRanking { ... }
oath: YAGNI
fix:  delete
```

### V022: Unused function completion_fraction
```
file: filmorator-core/src/matchup.rs:104-109
code: pub const fn completion_fraction(...) -> (u64, u64)
oath: YAGNI
fix:  delete
```

### V023: Unused column width
```
file: migrations/20250108_001_initial_schema.sql:6
code: width INT NOT NULL
oath: YAGNI
fix:  migration to drop
```

### V024: Unused column height
```
file: migrations/20250108_001_initial_schema.sql:7
code: height INT NOT NULL
oath: YAGNI
fix:  migration to drop
```

### V025: Unused column created_at (photos)
```
file: migrations/20250108_001_initial_schema.sql:10
code: created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
oath: YAGNI
fix:  migration to drop
```

### V026: Duplicated i32↔u32 conversion
```
file: filmorator-web/src/db.rs:40-44,90-94
code: .filter_map(|&idx| i32::try_from(idx).ok())
oath: DRY
fix:  u32_vec_to_i32_vec() helper
```

### V027: Duplicated matchup row mapping
```
file: filmorator-web/src/db.rs:75-86,227-238
code: Matchup { id: r.get(...), ... }
oath: DRY
fix:  matchup_from_row() helper
```

### V028: Duplicated error response pattern
```
file: filmorator-web/src/handlers/api.rs:64,71,124,192,208,216,224,253,261,283,321
code: (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
oath: DRY
fix:  AppError::Database(e)
```

### V029: Handler >20 lines (create_matchup)
```
file: filmorator-web/src/handlers/api.rs:55-170
code: 115 lines
oath: DEC
fix:  out of scope (refactor later)
```

### V030: Handler >20 lines (submit_comparison)
```
file: filmorator-web/src/handlers/api.rs:172-244
code: 72 lines
oath: DEC
fix:  out of scope (refactor later)
```

### V031: WHAT comment (Photo)
```
file: filmorator-core/src/models.rs:5
code: /// A photo in the collection.
oath: CS
fix:  delete
```

### V032: WHAT comment (Session)
```
file: filmorator-core/src/models.rs:18
code: /// An anonymous user session.
oath: CS
fix:  delete
```

### V033: WHAT comment (Matchup)
```
file: filmorator-core/src/models.rs:44
code: /// A matchup: group of photos shown together for ranking.
oath: CS
fix:  delete
```

### V034: WHAT comment (ComparisonResult)
```
file: filmorator-core/src/models.rs:69-70
code: /// User's ranking result for a matchup...
oath: CS
fix:  delete
```

### V035: WHAT comment (PhotoRating)
```
file: filmorator-core/src/models.rs:107
code: /// Bradley-Terry rating for a photo within a session.
oath: CS
fix:  delete
```

### V036: WHAT comment (presign_url)
```
file: filmorator-web/src/s3.rs:62
code: /// Generates a presigned URL for an image.
oath: CS
fix:  delete
```

---

## Cross-Reference Matrix

Violations discovered by multiple oaths:

| Violation | CbC | SME | DEC | YAGNI | DRY | CS |
|-----------|-----|-----|-----|-------|-----|----|
| V011 (unwrap_or count) | ✓ | | | | ✓ | |
| V016-18 (unwrap_or_default) | ✓ | | | | ✓ | |
| V019-20 (circular fallback) | ✓ | ✓ | | | | |
| V026-27 (dup conversion) | | | | | ✓ | |
| V028 (dup error response) | | | | | ✓ | |

---

## Severity Under CbC Governance

### Tier 1: Safety (must fix)
V001-V020 — Panics, silent failures, data corruption paths

### Tier 2: Waste (should fix)
V021-V025 — Dead code carrying maintenance burden

### Tier 3: Hygiene (fix if touching)
V026-V028 — DRY violations
V031-V036 — WHAT comments

### Tier 4: Deferred
V029-V030 — Handler length (refactor scope)

---

## Resolution Status

| ID | Status | Phase |
|----|--------|-------|
| V001-V004 | pending | 1 |
| V005-V010 | pending | 2 |
| V011-V020 | pending | 3 |
| V021-V025 | pending | 4 |
| V026-V028 | pending | 3,2 |
| V029-V030 | deferred | — |
| V031-V036 | pending | 5 |
