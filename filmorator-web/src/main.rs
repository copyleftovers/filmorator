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
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "filmorator_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("../migrations").run(&db).await?;

    let s3 = match &config.s3_endpoint {
        Some(endpoint) => {
            tracing::info!("Using custom S3 endpoint: {endpoint}");
            if let Some(public) = &config.s3_public_url {
                tracing::info!("Public S3 URL for presigning: {public}");
            }
            S3Client::with_endpoint(
                config.bucket.clone(),
                endpoint,
                config.s3_public_url.as_deref(),
            )
            .await
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
        .route("/img/:tier/:id", get(handlers::api::get_image))
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
