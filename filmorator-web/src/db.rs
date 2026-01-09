use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use filmorator_core::models::{ComparisonResult, Matchup, PhotoRating, Session};

pub async fn get_or_create_session(pool: &PgPool, session_id: Uuid) -> sqlx::Result<Session> {
    let now = Utc::now();

    let row = sqlx::query(
        r"
        INSERT INTO sessions (id, created_at, last_active_at)
        VALUES ($1, $2, $2)
        ON CONFLICT (id) DO UPDATE SET last_active_at = $2
        RETURNING id, created_at, last_active_at
        ",
    )
    .bind(session_id)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(Session {
        id: row.get("id"),
        created_at: row.get("created_at"),
        last_active_at: row.get("last_active_at"),
    })
}

pub async fn count_photos(pool: &PgPool) -> sqlx::Result<u32> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM photos")
        .fetch_one(pool)
        .await?;

    let count: i64 = row.get("count");
    Ok(u32::try_from(count).unwrap_or(0))
}

pub async fn create_matchup(pool: &PgPool, matchup: &Matchup) -> sqlx::Result<()> {
    let indices: Vec<i32> = matchup
        .photo_indices
        .iter()
        .filter_map(|&idx| i32::try_from(idx).ok())
        .collect();

    sqlx::query(
        r"
        INSERT INTO matchups (id, session_id, photo_indices, is_seed, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ",
    )
    .bind(matchup.id)
    .bind(matchup.session_id)
    .bind(&indices)
    .bind(matchup.is_seed)
    .bind(matchup.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_matchup(pool: &PgPool, matchup_id: Uuid) -> sqlx::Result<Option<Matchup>> {
    let row = sqlx::query(
        r"
        SELECT id, session_id, photo_indices, is_seed, created_at
        FROM matchups
        WHERE id = $1
        ",
    )
    .bind(matchup_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Matchup {
        id: r.get("id"),
        session_id: r.get("session_id"),
        photo_indices: r
            .get::<Option<Vec<i32>>, _>("photo_indices")
            .unwrap_or_default()
            .into_iter()
            .filter_map(|i| u32::try_from(i).ok())
            .collect(),
        is_seed: r.get("is_seed"),
        created_at: r.get("created_at"),
    }))
}

pub async fn save_comparison(pool: &PgPool, result: &ComparisonResult) -> sqlx::Result<()> {
    let ranked: Vec<i32> = result
        .ranked_photo_indices
        .iter()
        .filter_map(|&idx| i32::try_from(idx).ok())
        .collect();

    sqlx::query(
        r"
        INSERT INTO comparison_results (id, matchup_id, session_id, ranked_photo_indices, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ",
    )
    .bind(result.id)
    .bind(result.matchup_id)
    .bind(result.session_id)
    .bind(&ranked)
    .bind(result.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_session_comparisons(
    pool: &PgPool,
    session_id: Uuid,
) -> sqlx::Result<Vec<ComparisonResult>> {
    let rows = sqlx::query(
        r"
        SELECT id, matchup_id, session_id, ranked_photo_indices, created_at
        FROM comparison_results
        WHERE session_id = $1
        ORDER BY created_at
        ",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ComparisonResult {
            id: r.get("id"),
            matchup_id: r.get("matchup_id"),
            session_id: r.get("session_id"),
            ranked_photo_indices: r
                .get::<Option<Vec<i32>>, _>("ranked_photo_indices")
                .unwrap_or_default()
                .into_iter()
                .filter_map(|i| u32::try_from(i).ok())
                .collect(),
            created_at: r.get("created_at"),
        })
        .collect())
}

pub async fn get_session_ratings(
    pool: &PgPool,
    session_id: Uuid,
) -> sqlx::Result<Vec<PhotoRating>> {
    let rows = sqlx::query(
        r"
        SELECT photo_idx, strength, uncertainty
        FROM photo_ratings
        WHERE session_id = $1
        ORDER BY strength DESC
        ",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|r| {
            Some(PhotoRating {
                photo_idx: u32::try_from(r.get::<i32, _>("photo_idx")).ok()?,
                strength: r.get("strength"),
                uncertainty: r.get("uncertainty"),
            })
        })
        .collect())
}

pub async fn save_ratings(
    pool: &PgPool,
    session_id: Uuid,
    ratings: &[PhotoRating],
) -> sqlx::Result<()> {
    for rating in ratings {
        let photo_idx = i32::try_from(rating.photo_idx).unwrap_or(0);
        sqlx::query(
            r"
            INSERT INTO photo_ratings (session_id, photo_idx, strength, uncertainty)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (session_id, photo_idx) DO UPDATE
            SET strength = $3, uncertainty = $4
            ",
        )
        .bind(session_id)
        .bind(photo_idx)
        .bind(rating.strength)
        .bind(rating.uncertainty)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn count_compared_pairs(pool: &PgPool, session_id: Uuid) -> sqlx::Result<u64> {
    let comparisons = get_session_comparisons(pool, session_id).await?;
    let pairs: std::collections::HashSet<(u32, u32)> = comparisons
        .iter()
        .flat_map(ComparisonResult::to_pairwise)
        .map(|(a, b)| filmorator_core::matchup::normalize_pair(a, b))
        .collect();
    Ok(u64::try_from(pairs.len()).unwrap_or(0))
}

pub async fn get_pending_seed_matchup(
    pool: &PgPool,
    session_id: Uuid,
) -> sqlx::Result<Option<Matchup>> {
    let row = sqlx::query(
        r"
        SELECT m.id, m.session_id, m.photo_indices, m.is_seed, m.created_at
        FROM matchups m
        LEFT JOIN comparison_results cr ON m.id = cr.matchup_id
        WHERE m.session_id = $1 AND m.is_seed = true AND cr.id IS NULL
        ORDER BY m.created_at
        LIMIT 1
        ",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Matchup {
        id: r.get("id"),
        session_id: r.get("session_id"),
        photo_indices: r
            .get::<Option<Vec<i32>>, _>("photo_indices")
            .unwrap_or_default()
            .into_iter()
            .filter_map(|i| u32::try_from(i).ok())
            .collect(),
        is_seed: r.get("is_seed"),
        created_at: r.get("created_at"),
    }))
}

pub async fn has_seed_matchups(pool: &PgPool, session_id: Uuid) -> sqlx::Result<bool> {
    let row = sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM matchups WHERE session_id = $1 AND is_seed = true) as exists",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await?;

    Ok(row.get::<bool, _>("exists"))
}

pub async fn upsert_photo(pool: &PgPool, filename: &str, position: u32) -> sqlx::Result<()> {
    let id = Uuid::new_v4();
    let position_i32 = i32::try_from(position).unwrap_or(0);

    sqlx::query(
        r"
        INSERT INTO photos (id, filename, width, height, file_hash, position)
        VALUES ($1, $2, 0, 0, $2, $3)
        ON CONFLICT (file_hash) DO UPDATE SET position = $3
        ",
    )
    .bind(id)
    .bind(filename)
    .bind(position_i32)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_photo_filename_by_position(
    pool: &PgPool,
    position: u32,
) -> sqlx::Result<Option<String>> {
    let position_i32 = i32::try_from(position).unwrap_or(0);

    let row = sqlx::query("SELECT filename FROM photos WHERE position = $1")
        .bind(position_i32)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| r.get("filename")))
}
