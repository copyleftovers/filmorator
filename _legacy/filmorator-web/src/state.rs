use sqlx::PgPool;

use crate::s3::S3Client;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub s3: S3Client,
}

impl AppState {
    #[must_use]
    pub fn new(db: PgPool, s3: S3Client) -> Self {
        Self { db, s3 }
    }
}
