pub mod subscription_routes;
pub mod role_routes;

use axum::{routing::get, Router};
use crate::AppState;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(admin_health_check))
        .nest("/subscriptions", subscription_routes::subscription_admin_routes())
        .nest("/roles", role_routes::role_admin_routes())
}

async fn admin_health_check() -> &'static str {
    "Admin API is healthy"
}