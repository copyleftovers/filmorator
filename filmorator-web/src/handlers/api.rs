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
use crate::error::AppError;
use crate::state::AppState;

use super::session::{session_cookie_header, SessionId};
use filmorator_core::matchup::{
    extract_compared_pairs, generate_seed_matchups, select_dynamic_matchup,
};
use filmorator_core::models::{ComparisonResult, Matchup};
use filmorator_core::ranking::BradleyTerry;

const MATCHUP_SIZE: u32 = 3;
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
) -> Result<impl IntoResponse, AppError> {
    let session_id = session.0;

    db::get_or_create_session(&state.db, session_id).await?;

    let num_photos = db::count_photos(&state.db).await?;

    if num_photos < MATCHUP_SIZE {
        return Err(AppError::BadRequest("Not enough photos"));
    }

    // Check for pending seed matchup first
    if let Some(pending) = db::get_pending_seed_matchup(&state.db, session_id).await? {
        let response = MatchupResponse {
            matchup_id: pending.id,
            photo_indices: pending.photo_indices,
        };
        return Ok((
            [(header::SET_COOKIE, session_cookie_header(session_id)?)],
            Json(response),
        )
            .into_response());
    }

    // Generate seed matchups if none exist
    let has_seeds = db::has_seed_matchups(&state.db, session_id).await?;
    if !has_seeds {
        let seeds = generate_seed_matchups(num_photos, MATCHUP_SIZE as usize);
        for indices in seeds {
            let matchup = Matchup::new(session_id, indices, true);
            db::create_matchup(&state.db, &matchup).await?;
        }

        // Return first seed matchup
        if let Some(first) = db::get_pending_seed_matchup(&state.db, session_id).await? {
            let response = MatchupResponse {
                matchup_id: first.id,
                photo_indices: first.photo_indices,
            };
            return Ok((
                [(header::SET_COOKIE, session_cookie_header(session_id)?)],
                Json(response),
            )
                .into_response());
        }
    }

    // Seeds exhausted: generate dynamic matchup
    let ratings = db::get_session_ratings(&state.db, session_id).await?;

    let comparisons = db::get_session_comparisons(&state.db, session_id).await?;

    let pairs: Vec<(u32, u32)> = comparisons
        .iter()
        .flat_map(ComparisonResult::to_pairwise)
        .collect();
    let compared = extract_compared_pairs(&pairs);

    let photo_indices = if ratings.is_empty() {
        // No ratings yet, pick first few photos
        (0..MATCHUP_SIZE).collect()
    } else {
        match select_dynamic_matchup(&ratings, &compared, MATCHUP_SIZE as usize) {
            Some(indices) => indices,
            None => {
                return Ok((StatusCode::OK, "All pairs compared").into_response());
            }
        }
    };

    let matchup = Matchup::new(session_id, photo_indices.clone(), false);
    db::create_matchup(&state.db, &matchup).await?;

    let response = MatchupResponse {
        matchup_id: matchup.id,
        photo_indices,
    };

    Ok((
        [(header::SET_COOKIE, session_cookie_header(session_id)?)],
        Json(response),
    )
        .into_response())
}

pub async fn submit_comparison(
    State(state): State<AppState>,
    session: SessionId,
    Json(request): Json<CompareRequest>,
) -> Result<impl IntoResponse, AppError> {
    let session_id = session.0;

    // Validate matchup exists and belongs to session
    let matchup = match db::get_matchup(&state.db, request.matchup_id).await? {
        Some(m) if m.session_id == session_id => m,
        Some(_) => {
            return Err(AppError::Forbidden("Matchup belongs to different session"));
        }
        None => return Err(AppError::NotFound("Matchup not found")),
    };

    // Validate ranked indices match matchup
    let matchup_set: HashSet<u32> = matchup.photo_indices.iter().copied().collect();
    let ranked_set: HashSet<u32> = request.ranked_photo_indices.iter().copied().collect();
    if matchup_set != ranked_set {
        return Err(AppError::BadRequest("Invalid ranking"));
    }

    // Save comparison result
    let result =
        ComparisonResult::new(request.matchup_id, session_id, request.ranked_photo_indices);
    db::save_comparison(&state.db, &result).await?;

    // Recompute ratings
    let num_photos = db::count_photos(&state.db).await?;

    let comparisons = db::get_session_comparisons(&state.db, session_id).await?;

    let Some(mut bt) = BradleyTerry::new(num_photos as usize) else {
        return Err(AppError::Internal("Too many photos"));
    };

    let pairs: Vec<(u32, u32)> = comparisons
        .iter()
        .flat_map(ComparisonResult::to_pairwise)
        .collect();
    bt.record_comparisons(&pairs);

    let ratings = bt.compute_ratings(RATING_ITERATIONS);
    db::save_ratings(&state.db, session_id, &ratings).await?;
    Ok((StatusCode::OK, "Comparison recorded").into_response())
}

pub async fn get_progress(
    State(state): State<AppState>,
    session: SessionId,
) -> Result<impl IntoResponse, AppError> {
    let session_id = session.0;

    let num_photos = db::count_photos(&state.db).await?;

    let compared = db::count_compared_pairs(&state.db, session_id).await?;

    let total = filmorator_core::matchup::total_pairs_needed(num_photos);
    let percent = filmorator_core::matchup::completion_percent(compared, num_photos);

    Ok(Json(ProgressResponse {
        compared_pairs: compared,
        total_pairs: total,
        percent,
    })
    .into_response())
}

pub async fn get_ranking(
    State(state): State<AppState>,
    session: SessionId,
) -> Result<impl IntoResponse, AppError> {
    let session_id = session.0;

    let ratings = db::get_session_ratings(&state.db, session_id).await?;

    let rankings: Vec<RankingEntry> = ratings
        .into_iter()
        .map(|r| RankingEntry {
            photo_idx: r.photo_idx,
            strength: r.strength,
            uncertainty: r.uncertainty,
        })
        .collect();

    Ok(Json(RankingResponse { rankings }).into_response())
}

pub async fn get_image(
    State(state): State<AppState>,
    Path((tier, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    use crate::s3::ImageTier;
    use std::time::Duration;

    let Some(tier) = ImageTier::from_str(&tier) else {
        return Err(AppError::BadRequest("Invalid tier"));
    };

    // Parse id as position (u32)
    let Ok(position) = id.parse::<u32>() else {
        return Err(AppError::BadRequest("Invalid photo id"));
    };

    // Get filename from database
    let Some(filename) = db::get_photo_filename_by_position(&state.db, position).await? else {
        return Err(AppError::NotFound("Photo not found"));
    };

    // Generate presigned URL (15 minute expiry)
    let url = state
        .s3
        .presign_url(tier, &filename, Duration::from_secs(900))
        .await?;

    // Redirect to presigned URL
    Ok((StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, url)]).into_response())
}

pub async fn sync_photos(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let filenames = state.s3.list_photos().await?;

    let count = filenames.len();

    for (position, filename) in filenames.into_iter().enumerate() {
        let pos = u32::try_from(position).map_err(|_| AppError::Internal("Position overflow"))?;
        db::upsert_photo(&state.db, &filename, pos).await?;
    }

    Ok((StatusCode::OK, format!("Synced {count} photos")).into_response())
}
