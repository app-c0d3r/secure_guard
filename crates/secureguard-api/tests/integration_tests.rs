use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use secureguard_api::{create_app, database::Database};
use secureguard_shared::{CreateUserRequest, LoginRequest, RegisterAgentRequest, AuthResponse};
use serde_json::json;
use tower::ServiceExt;
use http_body_util::BodyExt;

async fn setup_test_app() -> axum::Router {
    let database_url = std::env::var("DATABASE_URL_TEST")
        .unwrap_or_else(|_| "postgresql://secureguard:password@localhost:5432/secureguard_test".to_string());
    
    let database = Database::new(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    create_app(database).await
}

#[tokio::test]
async fn test_health_check() {
    let app = setup_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["service"], "secureguard-api");
}

#[tokio::test]
async fn test_user_registration_endpoint() {
    let app = setup_test_app().await;
    
    let user_data = CreateUserRequest {
        username: "integration_test_user".to_string(),
        email: "integration@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&user_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(auth_response.user.username, "integration_test_user");
    assert_eq!(auth_response.user.email, "integration@test.com");
    assert!(!auth_response.token.is_empty());
}

#[tokio::test]
async fn test_user_login_endpoint() {
    let app = setup_test_app().await;
    
    // First register a user
    let user_data = CreateUserRequest {
        username: "login_test_user".to_string(),
        email: "login@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&user_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Then try to login
    let login_data = LoginRequest {
        username: "login_test_user".to_string(),
        password: "password123".to_string(),
    };
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(auth_response.user.username, "login_test_user");
    assert!(!auth_response.token.is_empty());
}

#[tokio::test]
async fn test_protected_route_without_token() {
    let app = setup_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_with_token() {
    let app = setup_test_app().await;
    
    // First register and get token
    let user_data = CreateUserRequest {
        username: "protected_test_user".to_string(),
        email: "protected@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&user_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    let body = axum::body::to_bytes(register_response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();
    
    // Use token for protected route
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/me")
                .header("Authorization", format!("Bearer {}", auth_response.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_agent_registration_endpoint() {
    let app = setup_test_app().await;
    
    // First register user and get token
    let user_data = CreateUserRequest {
        username: "agent_test_user".to_string(),
        email: "agent@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&user_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    let body = axum::body::to_bytes(register_response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();
    
    // Register agent
    let agent_data = RegisterAgentRequest {
        hardware_fingerprint: "integration-test-fingerprint".to_string(),
        os_info: json!({
            "os": "Windows 11",
            "version": "22H2",
            "architecture": "x64"
        }),
        version: "1.0.0".to_string(),
    };
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/agents/register")
                .header("Authorization", format!("Bearer {}", auth_response.token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&agent_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_error_handling() {
    let app = setup_test_app().await;
    
    // Test invalid JSON
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test duplicate user registration
    let user_data = CreateUserRequest {
        username: "duplicate_user".to_string(),
        email: "duplicate@test.com".to_string(),
        password: "password123".to_string(),
    };
    
    // First registration
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&user_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Second registration should fail
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&user_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}