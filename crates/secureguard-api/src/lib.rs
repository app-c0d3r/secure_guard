pub mod config;
pub mod database;
pub mod handlers;
pub mod services;
pub mod middleware;
pub mod routes;
pub mod websocket;

pub use routes::create_api_routes;

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use database::Database;

pub async fn create_app(database: Database) -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .nest("/api/v1", create_api_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(database)
}