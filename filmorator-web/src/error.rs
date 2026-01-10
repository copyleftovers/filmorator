use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

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
            Self::Database(e) => {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            Self::S3(e) => {
                tracing::error!("S3: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "S3 error")
            }
            Self::NotFound(m) => (StatusCode::NOT_FOUND, *m),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, *m),
            Self::Forbidden(m) => (StatusCode::FORBIDDEN, *m),
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, *m),
        };
        (status, msg).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::S3(e)
    }
}
