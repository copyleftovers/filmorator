mod db;
mod handlers;
mod s3;
mod state;

use axum::{routing::get, routing::post, Router};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()
        .expect("PORT must be a number");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bucket = std::env::var("FILMORATOR_BUCKET").expect("FILMORATOR_BUCKET must be set");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("../migrations").run(&db).await?;

    let s3 = if let Ok(endpoint) = std::env::var("AWS_ENDPOINT_URL") {
        tracing::info!("Using custom S3 endpoint: {endpoint}");
        S3Client::with_endpoint(bucket, &endpoint).await
    } else {
        S3Client::from_env(bucket).await
    };

    // Sync photos from S3 on startup
    if let Err(e) = sync_photos_from_s3(&db, &s3).await {
        tracing::warn!("Failed to sync photos from S3: {e}");
    }

    let state = AppState::new(db, s3);

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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("Listening on port {port}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn sync_photos_from_s3(db: &sqlx::PgPool, s3: &S3Client) -> Result<(), anyhow::Error> {
    let filenames = s3.list_photos().await?;
    tracing::info!("Found {} photos in S3", filenames.len());

    for (position, filename) in filenames.iter().enumerate() {
        db::upsert_photo(db, filename, u32::try_from(position).unwrap_or(0)).await?;
    }

    Ok(())
}
