# Oath Violation Remediation Plan

Execute phases 0–6 in order. Each phase must pass all gates before proceeding.

---

## Phase 0: Dependencies

Add to `filmorator-web/Cargo.toml` after line 22:

```toml
thiserror = { workspace = true }
```

**Gate:**
```bash
grep -q "thiserror" filmorator-web/Cargo.toml && echo PASS || echo FAIL
```

---

## Phase 1: Eliminate Panic Paths

### 1.1 Create `filmorator-web/src/config.rs`

```rust
use std::num::NonZeroU16;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: NonZeroU16,
    pub database_url: String,
    pub bucket: String,
    pub s3_endpoint: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("PORT invalid: {0}")]
    InvalidPort(String),
    #[error("DATABASE_URL required")]
    MissingDatabaseUrl,
    #[error("FILMORATOR_BUCKET required")]
    MissingBucket,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
        let port = port_str
            .parse::<u16>()
            .ok()
            .and_then(NonZeroU16::new)
            .ok_or_else(|| ConfigError::InvalidPort(port_str))?;

        Ok(Self {
            port,
            database_url: std::env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?,
            bucket: std::env::var("FILMORATOR_BUCKET").map_err(|_| ConfigError::MissingBucket)?,
            s3_endpoint: std::env::var("AWS_ENDPOINT_URL").ok(),
        })
    }
}
```

### 1.2 Create `filmorator-web/src/error.rs`

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}};

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    S3(anyhow::Error),
    NotFound(&'static str),
    BadRequest(&'static str),
    Forbidden(&'static str),
    Internal(&'static str),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            Self::Database(e) => { tracing::error!("DB: {e}"); (StatusCode::INTERNAL_SERVER_ERROR, "Database error") }
            Self::S3(e) => { tracing::error!("S3: {e}"); (StatusCode::INTERNAL_SERVER_ERROR, "S3 error") }
            Self::NotFound(m) => (StatusCode::NOT_FOUND, *m),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, *m),
            Self::Forbidden(m) => (StatusCode::FORBIDDEN, *m),
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, *m),
        };
        (status, msg).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self { Self::Database(e) }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self { Self::S3(e) }
}
```

### 1.3 Replace `filmorator-web/src/main.rs`

```rust
mod config;
mod db;
mod error;
mod handlers;
mod s3;
mod state;

use axum::{routing::get, routing::post, Router};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use s3::S3Client;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "filmorator_web=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let db = PgPoolOptions::new().max_connections(5).connect(&config.database_url).await?;
    sqlx::migrate!("../migrations").run(&db).await?;

    let s3 = match &config.s3_endpoint {
        Some(endpoint) => {
            tracing::info!("Using custom S3 endpoint: {endpoint}");
            S3Client::with_endpoint(config.bucket.clone(), endpoint).await
        }
        None => S3Client::from_env(config.bucket.clone()).await,
    };

    sync_photos_from_s3(&db, &s3).await?;

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/compare", get(handlers::compare::page))
        .route("/api/matchup", post(handlers::api::create_matchup))
        .route("/api/compare", post(handlers::api::submit_comparison))
        .route("/api/ranking", get(handlers::api::get_ranking))
        .route("/api/progress", get(handlers::api::get_progress))
        .route("/api/sync", post(handlers::api::sync_photos))
        .route("/img/{tier}/{id}", get(handlers::api::get_image))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(AppState::new(db, s3));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    tracing::info!("Listening on port {}", config.port);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn sync_photos_from_s3(db: &sqlx::PgPool, s3: &S3Client) -> anyhow::Result<()> {
    let filenames = s3.list_photos().await?;
    tracing::info!("Found {} photos in S3", filenames.len());
    for (pos, filename) in filenames.iter().enumerate() {
        let pos = u32::try_from(pos).map_err(|_| anyhow::anyhow!("Position overflow"))?;
        db::upsert_photo(db, filename, pos).await?;
    }
    Ok(())
}
```

### 1.4 Fix `filmorator-web/src/s3.rs`

Replace lines 62-88 (the `presign_url` method):

```rust
    pub async fn presign_url(
        &self,
        tier: ImageTier,
        filename: &str,
        expires_in: Duration,
    ) -> anyhow::Result<String> {
        let key = format!("{}/{filename}", tier.as_prefix());
        let presigning_config = PresigningConfig::builder().expires_in(expires_in).build()?;
        let presigned = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .presigned(presigning_config)
            .await?;
        Ok(presigned.uri().to_string())
    }
```

**Gate:**
```bash
grep -rn "\.expect(" filmorator-web/src/{main.rs,s3.rs,config.rs} 2>/dev/null | grep -v test && echo FAIL || echo PASS
```

---

## Phase 2: Eliminate Silent Failures

### 2.1 Update `filmorator-web/src/handlers/api.rs`

**Imports** — add at top:
```rust
use crate::error::AppError;
```

**Constants** — change line 21:
```rust
const MATCHUP_SIZE: u32 = 3;
```

**All handler signatures** — change return type to `Result<impl IntoResponse, AppError>`:

| Line | Function | New Signature |
|------|----------|---------------|
| 55 | `create_matchup` | `-> Result<impl IntoResponse, AppError>` |
| 172 | `submit_comparison` | `-> Result<impl IntoResponse, AppError>` |
| 246 | `get_progress` | `-> Result<impl IntoResponse, AppError>` |
| 276 | `get_ranking` | `-> Result<impl IntoResponse, AppError>` |
| 299 | `get_image` | `-> Result<impl IntoResponse, AppError>` |
| 342 | `sync_photos` | `-> Result<impl IntoResponse, AppError>` |

**Pattern replacements** — apply throughout:

| Old Pattern | New Pattern |
|-------------|-------------|
| `return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()` | `return Err(AppError::Database(e))` |
| `return (StatusCode::NOT_FOUND, "...").into_response()` | `return Err(AppError::NotFound("..."))` |
| `return (StatusCode::BAD_REQUEST, "...").into_response()` | `return Err(AppError::BadRequest("..."))` |
| `return (StatusCode::FORBIDDEN, "...").into_response()` | `return Err(AppError::Forbidden("..."))` |
| `if let Err(e) = db::...{ tracing::error!(...); }` | `db::...?;` |
| `.unwrap_or(false)` | `?` (propagate error) |
| `u32::try_from(MATCHUP_SIZE).unwrap_or(3)` | `MATCHUP_SIZE` |

**Line 93-96** — replace:
```rust
    let has_seeds = db::has_seed_matchups(&state.db, session_id).await?;
    if !has_seeds {
```

**Line 97-103** — replace seed creation loop:
```rust
        let seeds = generate_seed_matchups(num_photos, MATCHUP_SIZE as usize);
        for indices in seeds {
            let matchup = Matchup::new(session_id, indices, true);
            db::create_matchup(&state.db, &matchup).await?;
        }
```

**Line 238-241** — replace rating save:
```rust
    let ratings = bt.compute_ratings(RATING_ITERATIONS);
    db::save_ratings(&state.db, session_id, &ratings).await?;
    Ok((StatusCode::OK, "Comparison recorded").into_response())
```

**Line 353-359** — replace sync loop:
```rust
    for (position, filename) in filenames.into_iter().enumerate() {
        let pos = u32::try_from(position).map_err(|_| AppError::Internal("Position overflow"))?;
        db::upsert_photo(&state.db, &filename, pos).await?;
    }
    Ok((StatusCode::OK, format!("Synced {count} photos")).into_response())
```

**Wrap all success returns** in `Ok(...)`.

### 2.2 Update `filmorator-web/src/handlers/session.rs`

Add import:
```rust
use crate::error::AppError;
```

Replace lines 47-52:
```rust
pub fn session_cookie_header(session_id: Uuid) -> Result<HeaderValue, AppError> {
    let cookie = format!("{SESSION_COOKIE}={session_id}; Path=/; HttpOnly; SameSite=Lax; Max-Age=31536000");
    HeaderValue::from_str(&cookie).map_err(|_| AppError::Internal("Invalid cookie"))
}
```

Update callers in `api.rs` to use `session_cookie_header(session_id)?`.

**Gate:**
```bash
grep -n "tracing::error.*; *$" filmorator-web/src/handlers/api.rs && echo FAIL || echo PASS
```

---

## Phase 3: Fix Type Conversions

### 3.1 Add helpers to `filmorator-web/src/db.rs`

Insert after imports:

```rust
fn i32_vec_to_u32_vec(v: Vec<i32>) -> sqlx::Result<Vec<u32>> {
    v.into_iter()
        .map(|i| u32::try_from(i).map_err(|_| sqlx::Error::Protocol("Negative index".into())))
        .collect()
}

fn u32_vec_to_i32_vec(v: &[u32]) -> sqlx::Result<Vec<i32>> {
    v.iter()
        .map(|&i| i32::try_from(i).map_err(|_| sqlx::Error::Protocol("Index overflow".into())))
        .collect()
}

fn matchup_from_row(row: sqlx::postgres::PgRow) -> sqlx::Result<Matchup> {
    use sqlx::Row;
    Ok(Matchup {
        id: row.get("id"),
        session_id: row.get("session_id"),
        photo_indices: i32_vec_to_u32_vec(row.get("photo_indices"))?,
        is_seed: row.get("is_seed"),
        created_at: row.get("created_at"),
    })
}
```

### 3.2 Replacements

| Location | Old | New |
|----------|-----|-----|
| `count_photos` line 36 | `u32::try_from(count).unwrap_or(0)` | `u32::try_from(count).map_err(\|_\| sqlx::Error::Protocol("Count overflow".into()))?` |
| `create_matchup` lines 40-44 | filter_map chain | `let indices = u32_vec_to_i32_vec(&matchup.photo_indices)?;` |
| `get_matchup` lines 75-86 | inline mapping | `row.map(matchup_from_row).transpose()` |
| `save_comparison` lines 90-94 | filter_map chain | `let ranked = u32_vec_to_i32_vec(&result.ranked_photo_indices)?;` |
| `get_session_comparisons` lines 135-140 | unwrap_or_default + filter_map | `i32_vec_to_u32_vec(r.get("ranked_photo_indices"))?` |
| `get_session_ratings` lines 164-170 | filter_map with ok()? | explicit `u32::try_from(...).map_err(...)?` |
| `save_ratings` line 180 | `unwrap_or(0)` | `.map_err(\|_\| sqlx::Error::Protocol("Index overflow".into()))?` |
| `count_compared_pairs` line 206 | `unwrap_or(0)` | `pairs.len() as u64` (safe: usize ≤ u64) |
| `get_pending_seed_matchup` lines 227-238 | inline mapping | `row.map(matchup_from_row).transpose()` |
| `upsert_photo` line 254 | `unwrap_or(0)` | `.map_err(\|_\| sqlx::Error::Protocol("Position overflow".into()))?` |
| `get_photo_filename_by_position` line 276 | `unwrap_or(0)` | `.map_err(\|_\| sqlx::Error::Protocol("Position overflow".into()))?` |

**Gate:**
```bash
grep -n "unwrap_or(0)\|unwrap_or_default()" filmorator-web/src/db.rs && echo FAIL || echo PASS
```

---

## Phase 4: Delete Dead Code

### 4.1 `filmorator-core/src/models.rs`

Delete lines 128-135 (`SessionRanking` struct).

Delete `width`, `height`, `created_at` from `Photo` struct (lines 10-11, 15).

### 4.2 `filmorator-core/src/matchup.rs`

Delete lines 104-109 (`completion_fraction` function).

### 4.3 Create `migrations/20250110_002_remove_unused_columns.sql`

```sql
ALTER TABLE photos DROP COLUMN width;
ALTER TABLE photos DROP COLUMN height;
ALTER TABLE photos DROP COLUMN created_at;
```

### 4.4 Update `filmorator-web/src/db.rs` `upsert_photo`

Change SQL (around line 256):
```rust
    sqlx::query(
        r"INSERT INTO photos (id, filename, file_hash, position)
          VALUES ($1, $2, $2, $3)
          ON CONFLICT (file_hash) DO UPDATE SET position = $3",
    )
```

**Gate:**
```bash
grep -n "SessionRanking\|completion_fraction\|width\|height" filmorator-core/src/*.rs filmorator-web/src/db.rs && echo FAIL || echo PASS
```

---

## Phase 5: Remove WHAT Comments

### `filmorator-core/src/models.rs`

Delete these lines:

| Line | Content |
|------|---------|
| 5 | `/// A photo in the collection.` |
| 18 | `/// An anonymous user session.` |
| 44 | `/// A matchup: group of photos shown together for ranking.` |
| 49 | `/// Indices into the photo collection.` |
| 51 | `/// Whether this matchup was generated from snic seed.` |
| 69-70 | `/// User's ranking result...` |
| 76 | `/// Ordered from best (first) to worst (last).` |
| 93-94 | `/// Expand this ranking...` |
| 107 | `/// Bradley-Terry rating for a photo within a session.` |
| 111 | `/// Log-strength parameter (higher = better).` |
| 113 | `/// Uncertainty estimate (standard error).` |
| 122 | `// Start at neutral` |
| 123 | `// High initial uncertainty` |

### `filmorator-core/src/matchup.rs`

Delete lines 7-8, 38-39, 104, 111.

### `filmorator-web/src/s3.rs`

Delete lines 4, 7-12, 36, 44, 51, 62, 90.

**Gate:**
```bash
cargo doc --package filmorator-core 2>&1 | grep -i "warning.*missing" && echo FAIL || echo PASS
```

---

## Phase 6: Final Verification

```bash
cargo fmt --all
cargo clippy --all-targets -- -W clippy::pedantic
cargo test --all
```

**Gate:**
```bash
cargo fmt --check && cargo clippy --all-targets -- -W clippy::pedantic 2>&1 | grep "^error" && echo FAIL || echo PASS
cargo test --all 2>&1 | grep -E "FAILED|error\[" && echo FAIL || echo PASS
```

---

## Forbidden Patterns

```rust
// BANNED: panic paths
.expect("...")
.unwrap()

// BANNED: silent conversion failure
.unwrap_or(0)
.unwrap_or_default()

// BANNED: log-and-continue
if let Err(e) = x { tracing::error!(...); }

// BANNED: error-as-false
.unwrap_or(false)

// BANNED: empty fallback
.unwrap_or_else(|_| HeaderValue::from_static(""))

// BANNED: WHAT comments
/// A photo in the collection.

// BANNED: dead code retention
#[allow(dead_code)]
```

---

## Checklist

| Gate | Command | Expected |
|------|---------|----------|
| 0.1 | `grep -q "thiserror" filmorator-web/Cargo.toml` | exit 0 |
| 1.1 | `grep -rn "\.expect(" filmorator-web/src/{main,s3,config}.rs` | empty |
| 2.1 | `grep -n "tracing::error.*; *$" filmorator-web/src/handlers/api.rs` | empty |
| 3.1 | `grep -n "unwrap_or(0)" filmorator-web/src/db.rs` | empty |
| 4.1 | `grep -n "SessionRanking" filmorator-core/src/models.rs` | empty |
| 5.1 | `cargo doc --package filmorator-core` | no warnings |
| 6.1 | `cargo test --all` | 0 failures |

All gates must pass. If any fails, fix before proceeding.
