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
        handlers::register, handlers::login, handlers::me,
    )
)]
pub struct ApiDoc;

pub fn router(state: AppState, cors: CorsLayer) -> Router {
    let api = Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/me", get(handlers::me))
        .with_state(state);
    Router::new()
        .nest("/api/v1", api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
