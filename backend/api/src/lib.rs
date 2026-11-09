use axum::Router;
use sea_orm::DatabaseConnection;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: String,
}

#[derive(OpenApi)]
#[openapi(info(title = "Popravi.si API", version = "0.1.0"))]
pub struct ApiDoc;

pub fn router(state: AppState, cors: CorsLayer) -> Router {
    let api = Router::new().with_state(state);
    Router::new()
        .nest("/api/v1", api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
