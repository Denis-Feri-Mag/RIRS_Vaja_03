use crate::{error::ApiResult, extract::CurrentUser, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use service::{
    auth::{self, LoginInput, LoginResponse, RegisterInput, UserDto},
    categories::{self, CategoryDto},
    jobs::{self, CreateJobInput, JobDto, JobListItem, MatchingPage},
    providers::{self, ProviderProfileDto, ProviderProfileInput},
    quotes::{self, MyQuoteItem, QuoteDto, QuoteInput, QuoteSort},
};
use utoipa::IntoParams;
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

#[utoipa::path(get, path = "/api/v1/providers/me", security(("bearer" = [])), responses((status = 200, body = ProviderProfileDto)))]
pub async fn my_profile(State(s): State<AppState>, CurrentUser(u): CurrentUser) -> ApiResult<Json<ProviderProfileDto>> {
    Ok(Json(providers::get_mine(&s.db, &u).await?))
}

#[utoipa::path(put, path = "/api/v1/providers/me", security(("bearer" = [])), request_body = ProviderProfileInput, responses((status = 200, body = ProviderProfileDto)))]
pub async fn update_my_profile(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Json(input): Json<ProviderProfileInput>,
) -> ApiResult<Json<ProviderProfileDto>> {
    Ok(Json(providers::update_mine(&s.db, &u, input).await?))
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

#[derive(Deserialize, IntoParams)]
pub struct PageQuery {
    page: Option<u64>,
}

#[utoipa::path(get, path = "/api/v1/job-requests/matching", params(PageQuery), security(("bearer" = [])), responses((status = 200, body = MatchingPage)))]
pub async fn matching(State(s): State<AppState>, CurrentUser(u): CurrentUser, Query(q): Query<PageQuery>) -> ApiResult<Json<MatchingPage>> {
    Ok(Json(jobs::matching(&s.db, &u, q.page.unwrap_or(1)).await?))
}

#[utoipa::path(get, path = "/api/v1/job-requests/{id}", security(("bearer" = [])), responses((status = 200, body = JobDto)))]
pub async fn get_job(State(s): State<AppState>, CurrentUser(u): CurrentUser, Path(id): Path<Uuid>) -> ApiResult<Json<JobDto>> {
    Ok(Json(jobs::get(&s.db, &u, id).await?))
}

#[utoipa::path(post, path = "/api/v1/job-requests/{id}/quotes", security(("bearer" = [])), request_body = QuoteInput, responses((status = 201, body = QuoteDto), (status = 403), (status = 409)))]
pub async fn create_quote(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<QuoteInput>,
) -> ApiResult<(StatusCode, Json<QuoteDto>)> {
    Ok((StatusCode::CREATED, Json(quotes::create(&s.db, &u, id, input).await?)))
}

#[derive(Deserialize, IntoParams)]
pub struct SortQuery {
    sort: Option<QuoteSort>,
}

#[utoipa::path(get, path = "/api/v1/job-requests/{id}/quotes", params(SortQuery), security(("bearer" = [])), responses((status = 200, body = [QuoteDto]), (status = 403)))]
pub async fn job_quotes(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path(id): Path<Uuid>,
    Query(q): Query<SortQuery>,
) -> ApiResult<Json<Vec<QuoteDto>>> {
    Ok(Json(quotes::list_for_job(&s.db, &u, id, q.sort.unwrap_or_default()).await?))
}

#[utoipa::path(get, path = "/api/v1/job-requests/{id}/my-quote", security(("bearer" = [])), responses((status = 200, body = Option<QuoteDto>)))]
pub async fn my_quote(State(s): State<AppState>, CurrentUser(u): CurrentUser, Path(id): Path<Uuid>) -> ApiResult<Json<Option<QuoteDto>>> {
    Ok(Json(quotes::my_quote(&s.db, &u, id).await?))
}

#[utoipa::path(put, path = "/api/v1/quotes/{id}", security(("bearer" = [])), request_body = QuoteInput, responses((status = 200, body = QuoteDto)))]
pub async fn update_quote(
    State(s): State<AppState>,
    CurrentUser(u): CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<QuoteInput>,
) -> ApiResult<Json<QuoteDto>> {
    Ok(Json(quotes::update(&s.db, &u, id, input).await?))
}

#[utoipa::path(post, path = "/api/v1/quotes/{id}/withdraw", security(("bearer" = [])), responses((status = 200, body = QuoteDto)))]
pub async fn withdraw_quote(State(s): State<AppState>, CurrentUser(u): CurrentUser, Path(id): Path<Uuid>) -> ApiResult<Json<QuoteDto>> {
    Ok(Json(quotes::withdraw(&s.db, &u, id).await?))
}

#[utoipa::path(post, path = "/api/v1/quotes/{id}/accept", security(("bearer" = [])), responses((status = 200, body = QuoteDto), (status = 403), (status = 409)))]
pub async fn accept_quote(State(s): State<AppState>, CurrentUser(u): CurrentUser, Path(id): Path<Uuid>) -> ApiResult<Json<QuoteDto>> {
    Ok(Json(quotes::accept(&s.db, &u, id).await?))
}

#[utoipa::path(get, path = "/api/v1/quotes/mine", security(("bearer" = [])), responses((status = 200, body = [MyQuoteItem])))]
pub async fn my_quotes(State(s): State<AppState>, CurrentUser(u): CurrentUser) -> ApiResult<Json<Vec<MyQuoteItem>>> {
    Ok(Json(quotes::list_mine(&s.db, &u).await?))
}
