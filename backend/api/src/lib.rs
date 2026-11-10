mod error;
mod extract;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::DatabaseConnection;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: String,
}

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let c = openapi.components.get_or_insert_with(Default::default);
        c.add_security_scheme(
            "bearer",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build()),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Popravi.si API", version = "0.1.0"),
    modifiers(&BearerAuth),
    paths(
        handlers::register, handlers::login, handlers::me, handlers::categories,
        handlers::my_profile, handlers::update_my_profile,
        handlers::create_job, handlers::my_jobs, handlers::matching, handlers::get_job, handlers::job_history, handlers::change_status,
        handlers::create_quote, handlers::job_quotes, handlers::my_quote, handlers::update_quote, handlers::withdraw_quote, handlers::accept_quote, handlers::my_quotes,
    )
)]
pub struct ApiDoc;

pub fn router(state: AppState, cors: CorsLayer) -> Router {
    let api = Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/me", get(handlers::me))
        .route("/categories", get(handlers::categories))
        .route("/providers/me", get(handlers::my_profile).put(handlers::update_my_profile))
        .route("/job-requests", post(handlers::create_job))
        .route("/job-requests/mine", get(handlers::my_jobs))
        .route("/job-requests/matching", get(handlers::matching))
        .route("/job-requests/{id}", get(handlers::get_job))
        .route("/job-requests/{id}/history", get(handlers::job_history))
        .route("/job-requests/{id}/status", post(handlers::change_status))
        .route("/job-requests/{id}/quotes", get(handlers::job_quotes).post(handlers::create_quote))
        .route("/job-requests/{id}/my-quote", get(handlers::my_quote))
        .route("/quotes/mine", get(handlers::my_quotes))
        .route("/quotes/{id}", axum::routing::put(handlers::update_quote))
        .route("/quotes/{id}/withdraw", post(handlers::withdraw_quote))
        .route("/quotes/{id}/accept", post(handlers::accept_quote))
        .with_state(state);
    Router::new()
        .nest("/api/v1", api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
