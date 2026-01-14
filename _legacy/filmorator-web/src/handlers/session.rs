use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderValue, StatusCode},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

const SESSION_COOKIE: &str = "session_id";

/// Extracts or creates a session ID from cookies.
pub struct SessionId(pub Uuid);

#[async_trait]
impl FromRequestParts<AppState> for SessionId {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session_id = parts
            .headers
            .get(header::COOKIE)
            .and_then(|h| h.to_str().ok())
            .and_then(parse_session_cookie)
            .unwrap_or_else(Uuid::new_v4);

        Ok(SessionId(session_id))
    }
}

fn parse_session_cookie(cookies: &str) -> Option<Uuid> {
    for cookie in cookies.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie
            .strip_prefix(SESSION_COOKIE)
            .and_then(|s| s.strip_prefix('='))
        {
            return Uuid::parse_str(value.trim()).ok();
        }
    }
    None
}

/// Creates a Set-Cookie header value for the session.
pub fn session_cookie_header(session_id: Uuid) -> Result<HeaderValue, AppError> {
    let cookie =
        format!("{SESSION_COOKIE}={session_id}; Path=/; HttpOnly; SameSite=Lax; Max-Age=31536000");
    HeaderValue::from_str(&cookie).map_err(|_| AppError::Internal("Invalid cookie"))
}
