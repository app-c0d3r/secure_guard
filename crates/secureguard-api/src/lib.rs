pub mod config;
pub mod database;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod services;
pub mod telemetry;
pub mod websocket;

pub use routes::create_api_routes;

use axum::{routing::get, Router, http::StatusCode, response::Response};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use database::Database;

async fn metrics_handler() -> Response<String> {
    match telemetry::get_metrics() {
        Ok(metrics) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; version=0.0.4")
            .body(metrics)
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Failed to get metrics: {}", e))
            .unwrap(),
    }
}

pub async fn create_app(database: Database) -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/metrics", get(metrics_handler))
        .nest("/api/v1", create_api_routes())
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(telemetry::trace_middleware))
                .layer(CorsLayer::permissive()),
        )
        .with_state(database)
}
