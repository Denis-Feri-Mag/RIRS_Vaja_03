use crate::{error::ApiResult, extract::CurrentUser, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use service::{
    auth::{self, LoginInput, LoginResponse, RegisterInput, UserDto},
    categories::{self, CategoryDto},
    jobs::{self, CreateJobInput, JobDto, JobListItem},
};
use uuid::Uuid;

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

#[utoipa::path(get, path = "/api/v1/categories", responses((status = 200, body = [CategoryDto])))]
pub async fn categories(State(s): State<AppState>) -> ApiResult<Json<Vec<CategoryDto>>> {
    Ok(Json(categories::list_active(&s.db).await?))
}

#[utoipa::path(post, path = "/api/v1/job-requests", security(("bearer" = [])), request_body = CreateJobInput, responses((status = 201, body = JobDto)))]
pub async fn create_job(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Json(input): Json<CreateJobInput>,
) -> ApiResult<(StatusCode, Json<JobDto>)> {
    Ok((StatusCode::CREATED, Json(jobs::create(&s.db, &u, input).await?)))
}

#[utoipa::path(get, path = "/api/v1/job-requests/mine", security(("bearer" = [])), responses((status = 200, body = [JobListItem])))]
pub async fn my_jobs(State(s): State<AppState>, CurrentUser(u): CurrentUser) -> ApiResult<Json<Vec<JobListItem>>> {
    Ok(Json(jobs::list_mine(&s.db, &u).await?))
}

#[utoipa::path(get, path = "/api/v1/job-requests/{id}", security(("bearer" = [])), responses((status = 200, body = JobDto)))]
pub async fn get_job(State(s): State<AppState>, CurrentUser(u): CurrentUser, Path(id): Path<Uuid>) -> ApiResult<Json<JobDto>> {
    Ok(Json(jobs::get(&s.db, &u, id).await?))
}
