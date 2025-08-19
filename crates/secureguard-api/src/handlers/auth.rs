use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use secureguard_shared::{CreateUserRequest, LoginRequest, AuthResponse, User, SecureGuardError};
use crate::{
    database::Database,
    services::{user_service::{UserService, PasswordPolicy}, auth_service::AuthService},
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

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct PasswordChangeResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct PasswordPolicyResponse {
    pub policy: PasswordPolicyInfo,
}

#[derive(Serialize)]
pub struct PasswordPolicyInfo {
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: i32,
}

#[derive(Serialize)]
pub struct AuthStatusResponse {
    pub must_change_password: bool,
    pub user: User,
}

pub async fn change_password(
    AuthUser(user): AuthUser,
    State(db): State<Database>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<(StatusCode, Json<PasswordChangeResponse>), (StatusCode, Json<serde_json::Value>)> {
    let auth_service = AuthService::new("your-secret-key-change-in-production".to_string());
    let user_service = UserService::new(db.pool().clone(), auth_service);

    user_service
        .change_password(user.user_id, &request.old_password, &request.new_password)
        .await
        .map_err(|e| handle_error(e))?;

    let response = PasswordChangeResponse {
        success: true,
        message: "Password changed successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn password_policy(
    State(db): State<Database>,
) -> Result<(StatusCode, Json<PasswordPolicyResponse>), (StatusCode, Json<serde_json::Value>)> {
    let auth_service = AuthService::new("your-secret-key-change-in-production".to_string());
    let user_service = UserService::new(db.pool().clone(), auth_service);

    let policy = user_service
        .get_password_policy()
        .await
        .map_err(|e| handle_error(e))?;

    let response = PasswordPolicyResponse {
        policy: PasswordPolicyInfo {
            min_length: policy.min_length,
            require_uppercase: policy.require_uppercase,
            require_lowercase: policy.require_lowercase,
            require_numbers: policy.require_numbers,
            require_special_chars: policy.require_special_chars,
            max_age_days: policy.max_age_days,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn auth_status(
    AuthUser(user): AuthUser,
    State(db): State<Database>,
) -> Result<(StatusCode, Json<AuthStatusResponse>), (StatusCode, Json<serde_json::Value>)> {
    let auth_service = AuthService::new("your-secret-key-change-in-production".to_string());
    let user_service = UserService::new(db.pool().clone(), auth_service);

    let must_change = user_service
        .must_change_password(user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    let response = AuthStatusResponse {
        must_change_password: must_change,
        user,
    };

    Ok((StatusCode::OK, Json(response)))
}