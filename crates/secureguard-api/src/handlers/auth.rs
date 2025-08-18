use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use secureguard_shared::{CreateUserRequest, LoginRequest, AuthResponse, User, SecureGuardError};
use crate::{
    database::Database,
    services::{user_service::UserService, auth_service::AuthService},
    middleware::auth::AuthUser,
};

pub async fn register(
    State(db): State<Database>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<serde_json::Value>)> {
    let auth_service = AuthService::new("your-secret-key-change-in-production".to_string());
    let user_service = UserService::new(db.pool().clone(), auth_service.clone());

    let user = user_service.create_user(request).await
        .map_err(|e| handle_error(e))?;

    let token = auth_service.generate_token(user.user_id)
        .map_err(|e| handle_error(e))?;

    let response = AuthResponse { token, user };
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login(
    State(db): State<Database>,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<serde_json::Value>)> {
    let auth_service = AuthService::new("your-secret-key-change-in-production".to_string());
    let user_service = UserService::new(db.pool().clone(), auth_service.clone());

    let user = user_service.verify_credentials(&request.username, &request.password).await
        .map_err(|e| handle_error(e))?
        .ok_or_else(|| handle_error(SecureGuardError::AuthenticationFailed))?;

    let token = auth_service.generate_token(user.user_id)
        .map_err(|e| handle_error(e))?;

    let response = AuthResponse { token, user };
    Ok((StatusCode::OK, Json(response)))
}

pub async fn me(
    AuthUser(user): AuthUser,
) -> Json<User> {
    Json(user)
}

fn handle_error(error: SecureGuardError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, message) = match error {
        SecureGuardError::AuthenticationFailed => (StatusCode::UNAUTHORIZED, "Authentication failed".to_string()),
        SecureGuardError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        SecureGuardError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
    };

    (status, Json(serde_json::json!({ "error": message })))
}