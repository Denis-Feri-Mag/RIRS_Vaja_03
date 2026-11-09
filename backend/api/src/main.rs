use api::{router, AppState};
use axum::http::{header, HeaderValue, Method};
use migration::{Migrator, MigratorTrait};
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("manjka spremenljivka {name}"))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn".into()))
        .init();

    let db = sea_orm::Database::connect(env("DATABASE_URL")).await.expect("povezava z bazo");
    Migrator::up(&db, None).await.expect("migracije");
    service::seed::ensure_admin(&db, &env("ADMIN_EMAIL"), &env("ADMIN_PASSWORD"))
        .await
        .expect("admin");

    let origin = std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".into());
    let cors = CorsLayer::new()
        .allow_origin(origin.parse::<HeaderValue>().expect("CORS_ORIGIN"))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let state = AppState {
        db,
        jwt_secret: env("JWT_SECRET"),
    };
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    tracing::info!("API posluša na portu {port}");
    axum::serve(listener, router(state, cors)).await.unwrap();
}
