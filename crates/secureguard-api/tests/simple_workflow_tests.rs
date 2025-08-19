// Simple Workflow Tests for SecureGuard Application
// Tests core authentication and user management workflows

use secureguard_api::database::Database;
use secureguard_api::services::{
    auth_service::AuthService, 
    user_service::UserService,
};
use secureguard_shared::CreateUserRequest;
use uuid::Uuid;

// Test setup helper without borrowing issues
async fn create_test_setup() -> (Database, AuthService, UserService) {
    let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
        "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
    });
    
    let database = Database::new(&database_url)
        .await
        .expect("Failed to connect to test database");
        
    let auth_service = AuthService::new("test-secret-key-for-workflows".to_string());
    let user_service = UserService::new(database.pool().clone(), auth_service.clone());
    
    (database, auth_service, user_service)
}

async fn cleanup_test_data(database: &Database) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM users.users WHERE email LIKE '%@workflow-test.com'")
        .execute(database.pool()).await?;
    Ok(())
}

#[tokio::test]
async fn test_basic_user_registration() {
    let (database, auth_service, user_service) = create_test_setup().await;
    
    // Test user creation
    let create_request = CreateUserRequest {
        username: "test_user".to_string(),
        email: "test_user@workflow-test.com".to_string(),
        password: "TestPassword123!".to_string(),
    };
    
    let user = user_service.create_user(create_request).await
        .expect("User creation should succeed");
    
    assert_eq!(user.username, "test_user");
    assert_eq!(user.email, "test_user@workflow-test.com");
    assert!(user.is_active);
    
    cleanup_test_data(&database).await.expect("Cleanup should work");
}

#[tokio::test]
async fn test_user_authentication() {
    let (database, auth_service, user_service) = create_test_setup().await;
    
    // Create user
    let create_request = CreateUserRequest {
        username: "auth_user".to_string(),
        email: "auth_user@workflow-test.com".to_string(),
        password: "AuthPassword123!".to_string(),
    };
    
    let user = user_service.create_user(create_request).await
        .expect("User creation should succeed");
    
    // Test login
    let login_result = user_service
        .verify_credentials("auth_user", "AuthPassword123!")
        .await
        .expect("Login should work");
    
    assert!(login_result.is_some());
    let verified_user = login_result.unwrap();
    assert_eq!(verified_user.user_id, user.user_id);
    
    cleanup_test_data(&database).await.expect("Cleanup should work");
}

#[tokio::test]
async fn test_token_generation_and_validation() {
    let (database, auth_service, user_service) = create_test_setup().await;
    
    // Create user
    let create_request = CreateUserRequest {
        username: "token_user".to_string(),
        email: "token_user@workflow-test.com".to_string(),
        password: "TokenPassword123!".to_string(),
    };
    
    let user = user_service.create_user(create_request).await
        .expect("User creation should succeed");
    
    // Generate token
    let token = auth_service.generate_token(user.user_id)
        .expect("Token generation should work");
    
    // Validate token
    let claims = auth_service.verify_token(&token)
        .expect("Token validation should work");
    
    assert_eq!(claims.sub, user.user_id.to_string());
    
    cleanup_test_data(&database).await.expect("Cleanup should work");
}

#[tokio::test]
async fn test_invalid_login_attempts() {
    let (database, _auth_service, user_service) = create_test_setup().await;
    
    // Create user
    let create_request = CreateUserRequest {
        username: "secure_user".to_string(),
        email: "secure_user@workflow-test.com".to_string(),
        password: "SecurePassword123!".to_string(),
    };
    
    user_service.create_user(create_request).await
        .expect("User creation should succeed");
    
    // Test wrong password
    let wrong_pass_result = user_service
        .verify_credentials("secure_user", "WrongPassword")
        .await
        .expect("Should not error");
    assert!(wrong_pass_result.is_none());
    
    // Test non-existent user
    let no_user_result = user_service
        .verify_credentials("nonexistent", "SecurePassword123!")
        .await
        .expect("Should not error");
    assert!(no_user_result.is_none());
    
    cleanup_test_data(&database).await.expect("Cleanup should work");
}

#[tokio::test]
async fn test_duplicate_user_prevention() {
    let (database, _auth_service, user_service) = create_test_setup().await;
    
    // Create first user
    let create_request1 = CreateUserRequest {
        username: "duplicate_test".to_string(),
        email: "duplicate1@workflow-test.com".to_string(),
        password: "Password123!".to_string(),
    };
    
    let result1 = user_service.create_user(create_request1).await;
    assert!(result1.is_ok());
    
    // Try to create user with same username
    let create_request2 = CreateUserRequest {
        username: "duplicate_test".to_string(), // Same username
        email: "duplicate2@workflow-test.com".to_string(),
        password: "Password123!".to_string(),
    };
    
    let result2 = user_service.create_user(create_request2).await;
    assert!(result2.is_err()); // Should fail
    
    cleanup_test_data(&database).await.expect("Cleanup should work");
}

#[tokio::test]
async fn test_password_security() {
    let (database, auth_service, user_service) = create_test_setup().await;
    
    // Create user with strong password
    let create_request = CreateUserRequest {
        username: "secure_pass_user".to_string(),
        email: "secure_pass@workflow-test.com".to_string(),
        password: "VerySecurePassword123!@#".to_string(),
    };
    
    let user = user_service.create_user(create_request).await
        .expect("User creation should succeed");
    
    // Test correct password works
    let login_result = user_service
        .verify_credentials("secure_pass_user", "VerySecurePassword123!@#")
        .await
        .expect("Login should work");
    assert!(login_result.is_some());
    
    // Test wrong password fails
    let wrong_result = user_service
        .verify_credentials("secure_pass_user", "WrongPassword")
        .await
        .expect("Should not error");
    assert!(wrong_result.is_none());
    
    cleanup_test_data(&database).await.expect("Cleanup should work");
}

#[tokio::test]
async fn test_multiple_users_creation() {
    let (database, _auth_service, user_service) = create_test_setup().await;
    
    let usernames = vec!["user1", "user2", "user3", "user4"];
    let mut created_users = Vec::new();
    
    // Create multiple users
    for username in &usernames {
        let create_request = CreateUserRequest {
            username: username.to_string(),
            email: format!("{}@workflow-test.com", username),
            password: "MultiUser123!".to_string(),
        };
        
        let user = user_service.create_user(create_request).await
            .expect(&format!("User creation should succeed for {}", username));
        created_users.push(user);
    }
    
    // Verify all users were created
    assert_eq!(created_users.len(), 4);
    
    // Verify all users can log in
    for (i, username) in usernames.iter().enumerate() {
        let login_result = user_service
            .verify_credentials(username, "MultiUser123!")
            .await
            .expect("Login should work");
        assert!(login_result.is_some());
        
        let verified_user = login_result.unwrap();
        assert_eq!(verified_user.user_id, created_users[i].user_id);
    }
    
    cleanup_test_data(&database).await.expect("Cleanup should work");
}

#[tokio::test]
async fn test_token_validation_edge_cases() {
    let (_database, auth_service, _user_service) = create_test_setup().await;
    
    // Test invalid tokens
    assert!(auth_service.verify_token("").is_err());
    assert!(auth_service.verify_token("invalid").is_err());
    assert!(auth_service.verify_token("not.a.jwt.token").is_err());
    assert!(auth_service.verify_token("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.invalid.signature").is_err());
    
    // Valid token structure but wrong signature should fail
    let fake_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0IiwiaWF0IjoxNjc5NDE1NjAwLCJleHAiOjE2Nzk0MTkyMDB9.invalid_signature";
    assert!(auth_service.verify_token(fake_token).is_err());
}

#[tokio::test]
async fn test_database_connection() {
    let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
        "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
    });
    
    // Test database connection
    let database = Database::new(&database_url).await;
    assert!(database.is_ok());
    
    let db = database.unwrap();
    
    // Test basic query
    let result = sqlx::query!("SELECT 1 as test_value")
        .fetch_one(db.pool())
        .await;
    
    assert!(result.is_ok());
    let row = result.unwrap();
    assert_eq!(row.test_value.unwrap(), 1);
}

#[tokio::test]
async fn test_user_data_integrity() {
    let (database, _auth_service, user_service) = create_test_setup().await;
    
    // Create user
    let create_request = CreateUserRequest {
        username: "integrity_user".to_string(),
        email: "integrity@workflow-test.com".to_string(),
        password: "IntegrityTest123!".to_string(),
    };
    
    let user = user_service.create_user(create_request).await
        .expect("User creation should succeed");
    
    // Verify user data
    let login_result = user_service
        .verify_credentials("integrity_user", "IntegrityTest123!")
        .await
        .expect("Login should work")
        .expect("User should exist");
    
    // Check data integrity
    assert_eq!(login_result.user_id, user.user_id);
    assert_eq!(login_result.username, user.username);
    assert_eq!(login_result.email, user.email);
    assert_eq!(login_result.is_active, user.is_active);
    assert!(login_result.created_at <= login_result.updated_at);
    
    cleanup_test_data(&database).await.expect("Cleanup should work");
}

// Integration test covering complete workflow
#[tokio::test]
async fn test_complete_workflow_integration() {
    let (database, auth_service, user_service) = create_test_setup().await;
    
    println!("🚀 Starting complete workflow integration test");
    
    // Step 1: User Registration
    println!("📝 Testing user registration...");
    let create_request = CreateUserRequest {
        username: "integration_test".to_string(),
        email: "integration@workflow-test.com".to_string(),
        password: "IntegrationTest123!".to_string(),
    };
    
    let user = user_service.create_user(create_request).await
        .expect("User creation should succeed");
    
    assert!(!user.user_id.to_string().is_empty());
    assert_eq!(user.username, "integration_test");
    println!("✅ User registration successful");
    
    // Step 2: Authentication
    println!("🔐 Testing authentication...");
    let login_result = user_service
        .verify_credentials("integration_test", "IntegrationTest123!")
        .await
        .expect("Login should work")
        .expect("User should exist");
    
    assert_eq!(login_result.user_id, user.user_id);
    println!("✅ Authentication successful");
    
    // Step 3: Token Operations
    println!("🎫 Testing token operations...");
    let token = auth_service.generate_token(user.user_id)
        .expect("Token generation should work");
    
    let claims = auth_service.verify_token(&token)
        .expect("Token validation should work");
    
    assert_eq!(claims.sub, user.user_id.to_string());
    println!("✅ Token operations successful");
    
    // Step 4: Security Validation
    println!("🛡️ Testing security validations...");
    let wrong_login = user_service
        .verify_credentials("integration_test", "WrongPassword")
        .await
        .expect("Should not error");
    assert!(wrong_login.is_none());
    
    assert!(auth_service.verify_token("invalid_token").is_err());
    println!("✅ Security validations passed");
    
    // Step 5: Data Integrity
    println!("📊 Testing data integrity...");
    assert_eq!(login_result.username, user.username);
    assert_eq!(login_result.email, user.email);
    assert!(login_result.is_active);
    println!("✅ Data integrity verified");
    
    // Step 6: Multiple Sessions
    println!("🔄 Testing multiple sessions...");
    let token2 = auth_service.generate_token(user.user_id)
        .expect("Second token should generate");
    
    let claims1 = auth_service.verify_token(&token).expect("First token valid");
    let claims2 = auth_service.verify_token(&token2).expect("Second token valid");
    
    assert_eq!(claims1.sub, claims2.sub);
    println!("✅ Multiple sessions supported");
    
    // Cleanup
    cleanup_test_data(&database).await.expect("Cleanup should work");
    
    println!("🎉 Complete workflow integration test PASSED!");
}