use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::db;
use crate::state::AppState;

use super::session::{session_cookie_header, SessionId};
use filmorator_core::matchup::{
    extract_compared_pairs, generate_seed_matchups, select_dynamic_matchup,
};
use filmorator_core::models::{ComparisonResult, Matchup};
use filmorator_core::ranking::BradleyTerry;

const MATCHUP_SIZE: usize = 3;
const RATING_ITERATIONS: u32 = 50;

#[derive(Serialize)]
pub struct MatchupResponse {
    pub matchup_id: Uuid,
    pub photo_indices: Vec<u32>,
}

#[derive(Deserialize)]
pub struct CompareRequest {
    pub matchup_id: Uuid,
    pub ranked_photo_indices: Vec<u32>,
}

#[derive(Serialize)]
pub struct ProgressResponse {
    pub compared_pairs: u64,
    pub total_pairs: u64,
    pub percent: u8,
}

#[derive(Serialize)]
pub struct RankingResponse {
    pub rankings: Vec<RankingEntry>,
}

#[derive(Serialize)]
pub struct RankingEntry {
    pub photo_idx: u32,
    pub strength: f64,
    pub uncertainty: f64,
}

pub async fn create_matchup(
    State(state): State<AppState>,
    session: SessionId,
) -> impl IntoResponse {
    let session_id = session.0;

    let session_result = db::get_or_create_session(&state.db, session_id).await;
    if let Err(e) = session_result {
        tracing::error!("Failed to create session: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
    }

    let num_photos = match db::count_photos(&state.db).await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Failed to count photos: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    if num_photos < u32::try_from(MATCHUP_SIZE).unwrap_or(3) {
        return (StatusCode::BAD_REQUEST, "Not enough photos").into_response();
    }

    // Check for pending seed matchup first
    if let Ok(Some(pending)) = db::get_pending_seed_matchup(&state.db, session_id).await {
        let response = MatchupResponse {
            matchup_id: pending.id,
            photo_indices: pending.photo_indices,
        };
        return (
            [(header::SET_COOKIE, session_cookie_header(session_id))],
            Json(response),
        )
            .into_response();
    }

    // Generate seed matchups if none exist
    if !db::has_seed_matchups(&state.db, session_id)
        .await
        .unwrap_or(false)
    {
        let seeds = generate_seed_matchups(num_photos, MATCHUP_SIZE);
        for indices in seeds {
            let matchup = Matchup::new(session_id, indices, true);
            if let Err(e) = db::create_matchup(&state.db, &matchup).await {
                tracing::error!("Failed to save seed matchup: {e}");
            }
        }

        // Return first seed matchup
        if let Ok(Some(first)) = db::get_pending_seed_matchup(&state.db, session_id).await {
            let response = MatchupResponse {
                matchup_id: first.id,
                photo_indices: first.photo_indices,
            };
            return (
                [(header::SET_COOKIE, session_cookie_header(session_id))],
                Json(response),
            )
                .into_response();
        }
    }

    // Seeds exhausted: generate dynamic matchup
    let ratings = match db::get_session_ratings(&state.db, session_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to get ratings: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let comparisons = match db::get_session_comparisons(&state.db, session_id).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to get comparisons: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let pairs: Vec<(u32, u32)> = comparisons
        .iter()
        .flat_map(ComparisonResult::to_pairwise)
        .collect();
    let compared = extract_compared_pairs(&pairs);

    let photo_indices = if ratings.is_empty() {
        // No ratings yet, pick first few photos
        (0..u32::try_from(MATCHUP_SIZE).unwrap_or(3)).collect()
    } else {
        match select_dynamic_matchup(&ratings, &compared, MATCHUP_SIZE) {
            Some(indices) => indices,
            None => {
                return (StatusCode::OK, "All pairs compared").into_response();
            }
        }
    };

    let matchup = Matchup::new(session_id, photo_indices.clone(), false);
    if let Err(e) = db::create_matchup(&state.db, &matchup).await {
        tracing::error!("Failed to save matchup: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
    }

    let response = MatchupResponse {
        matchup_id: matchup.id,
        photo_indices,
    };

    (
        [(header::SET_COOKIE, session_cookie_header(session_id))],
        Json(response),
    )
        .into_response()
}

pub async fn submit_comparison(
    State(state): State<AppState>,
    session: SessionId,
    Json(request): Json<CompareRequest>,
) -> impl IntoResponse {
    let session_id = session.0;

    // Validate matchup exists and belongs to session
    let matchup = match db::get_matchup(&state.db, request.matchup_id).await {
        Ok(Some(m)) if m.session_id == session_id => m,
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                "Matchup belongs to different session",
            )
                .into_response()
        }
        Ok(None) => return (StatusCode::NOT_FOUND, "Matchup not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get matchup: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Validate ranked indices match matchup
    let matchup_set: HashSet<u32> = matchup.photo_indices.iter().copied().collect();
    let ranked_set: HashSet<u32> = request.ranked_photo_indices.iter().copied().collect();
    if matchup_set != ranked_set {
        return (StatusCode::BAD_REQUEST, "Invalid ranking").into_response();
    }

    // Save comparison result
    let result =
        ComparisonResult::new(request.matchup_id, session_id, request.ranked_photo_indices);
    if let Err(e) = db::save_comparison(&state.db, &result).await {
        tracing::error!("Failed to save comparison: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
    }

    // Recompute ratings
    let num_photos = match db::count_photos(&state.db).await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Failed to count photos: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let comparisons = match db::get_session_comparisons(&state.db, session_id).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to get comparisons: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let Some(mut bt) = BradleyTerry::new(num_photos as usize) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Too many photos").into_response();
    };

    let pairs: Vec<(u32, u32)> = comparisons
        .iter()
        .flat_map(ComparisonResult::to_pairwise)
        .collect();
    bt.record_comparisons(&pairs);

    let ratings = bt.compute_ratings(RATING_ITERATIONS);
    if let Err(e) = db::save_ratings(&state.db, session_id, &ratings).await {
        tracing::error!("Failed to save ratings: {e}");
    }

    (StatusCode::OK, "Comparison recorded").into_response()
}

pub async fn get_progress(State(state): State<AppState>, session: SessionId) -> impl IntoResponse {
    let session_id = session.0;

    let num_photos = match db::count_photos(&state.db).await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Failed to count photos: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let compared = match db::count_compared_pairs(&state.db, session_id).await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Failed to count pairs: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let total = filmorator_core::matchup::total_pairs_needed(num_photos);
    let percent = filmorator_core::matchup::completion_percent(compared, num_photos);

    Json(ProgressResponse {
        compared_pairs: compared,
        total_pairs: total,
        percent,
    })
    .into_response()
}

pub async fn get_ranking(State(state): State<AppState>, session: SessionId) -> impl IntoResponse {
    let session_id = session.0;

    let ratings = match db::get_session_ratings(&state.db, session_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to get ratings: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let rankings: Vec<RankingEntry> = ratings
        .into_iter()
        .map(|r| RankingEntry {
            photo_idx: r.photo_idx,
            strength: r.strength,
            uncertainty: r.uncertainty,
        })
        .collect();

    Json(RankingResponse { rankings }).into_response()
}

pub async fn get_image(
    State(state): State<AppState>,
    Path((tier, id)): Path<(String, String)>,
) -> impl IntoResponse {
    use crate::s3::ImageTier;
    use std::time::Duration;

    let Some(tier) = ImageTier::from_str(&tier) else {
        return (StatusCode::BAD_REQUEST, "Invalid tier").into_response();
    };

    // Parse id as position (u32)
    let Ok(position) = id.parse::<u32>() else {
        return (StatusCode::BAD_REQUEST, "Invalid photo id").into_response();
    };

    // Get filename from database
    let filename = match db::get_photo_filename_by_position(&state.db, position).await {
        Ok(Some(f)) => f,
        Ok(None) => return (StatusCode::NOT_FOUND, "Photo not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get photo: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Generate presigned URL (15 minute expiry)
    let url = match state
        .s3
        .presign_url(tier, &filename, Duration::from_secs(900))
        .await
    {
        Ok(url) => url,
        Err(e) => {
            tracing::error!("Failed to generate presigned URL: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "S3 error").into_response();
        }
    };

    // Redirect to presigned URL
    (StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, url)]).into_response()
}

pub async fn sync_photos(State(state): State<AppState>) -> impl IntoResponse {
    let filenames = match state.s3.list_photos().await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Failed to list photos from S3: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "S3 error").into_response();
        }
    };

    let count = filenames.len();

    for (position, filename) in filenames.into_iter().enumerate() {
        if let Err(e) =
            db::upsert_photo(&state.db, &filename, u32::try_from(position).unwrap_or(0)).await
        {
            tracing::error!("Failed to upsert photo {filename}: {e}");
        }
    }

    (StatusCode::OK, format!("Synced {count} photos")).into_response()
}
