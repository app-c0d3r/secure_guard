use crate::{
    database::Database,
    services::{auth_service::AuthService, user_service::UserService},
};
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use secureguard_shared::{SecureGuardError, User};
use uuid::Uuid;

pub struct AuthUser(pub User);

#[async_trait]
impl FromRequestParts<Database> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Database,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer "));

        let token = auth_header.ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing authorization header" })),
            )
        })?;

        let auth_service = AuthService::new("your-secret-key-change-in-production".to_string());
        let user_service = UserService::new(state.pool().clone(), auth_service.clone());

        let claims = auth_service.verify_token(token).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid token" })),
            )
        })?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid user ID in token" })),
            )
        })?;

        let user = user_service
            .find_by_id(user_id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Database error" })),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "User not found" })),
                )
            })?;

        Ok(AuthUser(user))
    }
}
