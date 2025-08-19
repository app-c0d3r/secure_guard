use secureguard_api::database::Database;
use secureguard_api::services::{auth_service::AuthService, user_service::UserService};
use secureguard_shared::{CreateUserRequest, LoginRequest};
use uuid::Uuid;

#[tokio::test]
async fn test_password_hashing() {
    let auth_service = AuthService::new("test-secret-key".to_string());

    let password = "test_password_123";
    let hash = auth_service.hash_password(password).unwrap();

    // Hash should be different from password
    assert_ne!(hash, password);
    assert!(hash.len() > 50); // Argon2 hashes are long

    // Should verify correctly
    assert!(auth_service.verify_password(password, &hash).unwrap());

    // Should fail with wrong password
    assert!(!auth_service
        .verify_password("wrong_password", &hash)
        .unwrap());
}

#[tokio::test]
async fn test_jwt_token_generation_and_validation() {
    let auth_service = AuthService::new("test-secret-key".to_string());
    let user_id = Uuid::new_v4();

    // Generate token
    let token = auth_service.generate_token(user_id).unwrap();
    assert!(!token.is_empty());

    // Verify token
    let claims = auth_service.verify_token(&token).unwrap();
    assert_eq!(claims.sub, user_id.to_string());

    // Invalid token should fail
    assert!(auth_service.verify_token("invalid_token").is_err());
    assert!(auth_service.verify_token("").is_err());
}

#[tokio::test]
async fn test_user_registration_and_login() {
    let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
        "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
    });
    let database = Database::new(&database_url)
        .await
        .expect("Failed to connect to test database");
    let auth_service = AuthService::new("test-secret-key".to_string());
    let user_service = UserService::new(database.pool().clone(), auth_service.clone());

    // Test user registration
    let create_request = CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let user = user_service.create_user(create_request).await.unwrap();
    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert!(user.is_active);

    // Verify user count
    // User created successfully

    // Test login with correct credentials
    let login_user = user_service
        .verify_credentials("testuser", "password123")
        .await
        .unwrap();
    assert!(login_user.is_some());
    assert_eq!(login_user.unwrap().user_id, user.user_id);

    // Test login with wrong password
    let invalid_login = user_service
        .verify_credentials("testuser", "wrong_password")
        .await
        .unwrap();
    assert!(invalid_login.is_none());

    // Test login with non-existent user
    let no_user = user_service
        .verify_credentials("nonexistent", "password123")
        .await
        .unwrap();
    assert!(no_user.is_none());
}

#[tokio::test]
async fn test_user_registration_validation() {
    let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
        "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
    });
    let database = Database::new(&database_url)
        .await
        .expect("Failed to connect to test database");
    let auth_service = AuthService::new("test-secret-key".to_string());
    let user_service = UserService::new(database.pool().clone(), auth_service);

    // Test empty username
    let invalid_request = CreateUserRequest {
        username: "".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };
    assert!(user_service.create_user(invalid_request).await.is_err());

    // Test short password
    let invalid_request = CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "short".to_string(),
    };
    assert!(user_service.create_user(invalid_request).await.is_err());

    // Test empty email
    let invalid_request = CreateUserRequest {
        username: "testuser".to_string(),
        email: "".to_string(),
        password: "password123".to_string(),
    };
    assert!(user_service.create_user(invalid_request).await.is_err());
}

#[tokio::test]
async fn test_duplicate_user_registration() {
    let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
        "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
    });
    let database = Database::new(&database_url)
        .await
        .expect("Failed to connect to test database");
    let auth_service = AuthService::new("test-secret-key".to_string());
    let user_service = UserService::new(database.pool().clone(), auth_service);

    let create_request = CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    // First registration should succeed
    user_service
        .create_user(create_request.clone())
        .await
        .unwrap();

    // Second registration with same username should fail
    assert!(user_service.create_user(create_request).await.is_err());
}
