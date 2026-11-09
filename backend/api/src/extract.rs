use crate::{error::ApiError, AppState};
use axum::{extract::FromRequestParts, http::request::Parts};
use service::AppError;

/// Logged in user, loaded from the JWT in the Authorization header.
pub struct CurrentUser(pub entity::user::Model);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::unauthorized("Prijava je obvezna."))?;
        let u = service::auth::current_user(&state.db, &state.jwt_secret, token).await?;
        Ok(CurrentUser(u))
    }
}
