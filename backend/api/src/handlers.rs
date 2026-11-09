use crate::{error::ApiResult, extract::CurrentUser, AppState};
use axum::{extract::State, http::StatusCode, Json};
use service::auth::{self, LoginInput, LoginResponse, RegisterInput, UserDto};

#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterInput, responses((status = 201, body = UserDto), (status = 409)))]
pub async fn register(State(s): State<AppState>, Json(input): Json<RegisterInput>) -> ApiResult<(StatusCode, Json<UserDto>)> {
    Ok((StatusCode::CREATED, Json(auth::register(&s.db, input).await?)))
}

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginInput, responses((status = 200, body = LoginResponse), (status = 401)))]
pub async fn login(State(s): State<AppState>, Json(input): Json<LoginInput>) -> ApiResult<Json<LoginResponse>> {
    Ok(Json(auth::login(&s.db, &s.jwt_secret, input).await?))
}

#[utoipa::path(get, path = "/api/v1/auth/me", security(("bearer" = [])), responses((status = 200, body = UserDto)))]
pub async fn me(CurrentUser(u): CurrentUser) -> Json<UserDto> {
    Json(u.into())
}
