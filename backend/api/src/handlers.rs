use crate::{error::ApiResult, AppState};
use axum::{extract::State, http::StatusCode, Json};
use service::auth::{self, RegisterInput, UserDto};

#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterInput, responses((status = 201, body = UserDto), (status = 409)))]
pub async fn register(State(s): State<AppState>, Json(input): Json<RegisterInput>) -> ApiResult<(StatusCode, Json<UserDto>)> {
    Ok((StatusCode::CREATED, Json(auth::register(&s.db, input).await?)))
}
