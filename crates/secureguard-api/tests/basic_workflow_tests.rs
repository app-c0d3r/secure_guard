// Basic Workflow Tests for SecureGuard Application
// Tests core authentication, user management, and basic workflows

use secureguard_api::database::Database;
use secureguard_api::services::{
    auth_service::AuthService, 
    user_service::UserService,
};
use secureguard_shared::CreateUserRequest;
use uuid::Uuid;
use std::collections::HashMap;

// Test helper struct for managing test users
#[derive(Debug, Clone)]
pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub auth_token: String,
}

// Test setup helper
pub struct TestSetup {
    pub database: Database,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub test_users: HashMap<String, TestUser>,
}

impl TestSetup {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
            "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
        });
        
        let database = Database::new(&database_url)
            .await
            .expect("Failed to connect to test database");
            
        let auth_service = AuthService::new("test-secret-key-for-workflows".to_string());
        let user_service = UserService::new(database.pool().clone(), auth_service.clone());
        
        TestSetup {
            database,
            auth_service,
            user_service,
            test_users: HashMap::new(),
        }
    }
    
    pub async fn create_test_user(&mut self, username: &str) -> TestUser {
        let unique_id = Uuid::new_v4().to_string()[..8].to_string();
        let unique_username = format!("{}_{}", username, unique_id);
        let email = format!("{}@workflow-test.com", unique_username);
        let password = "TestWorkflow123!";
        
        // Create user
        let create_request = CreateUserRequest {
            username: unique_username.clone(),
            email: email.clone(),
            password: password.to_string(),
        };
        
        let user = self.user_service.create_user(create_request).await
            .expect(&format!("Failed to create test user: {}", username));
            
        // Generate auth token
        let auth_token = self.auth_service.generate_token(user.user_id)
            .expect("Failed to generate auth token");
        
        let test_user = TestUser {
            user_id: user.user_id,
            username: unique_username.clone(),
            email,
            auth_token,
        };
        
        self.test_users.insert(username.to_string(), test_user.clone());
        test_user
    }
    
    pub async fn cleanup_test_data(&self) -> Result<(), sqlx::Error> {
        // Clean up test users
        sqlx::query!("DELETE FROM users.users WHERE email LIKE '%@workflow-test.com'")
            .execute(self.database.pool()).await?;
        
        Ok(())
    }
}

// BASIC AUTHENTICATION WORKFLOW TESTS
#[tokio::test]
async fn test_user_registration_and_login_workflow() {
    let mut setup = TestSetup::new().await;
    
    // Create test user
    let user = setup.create_test_user("workflow_user").await;
    
    // Test 1: Verify user was created
    assert!(!user.user_id.to_string().is_empty());
    assert_eq!(user.username, "workflow_user");
    assert_eq!(user.email, "workflow_user@workflow-test.com");
    
    // Test 2: Verify login works
    let login_check = setup.user_service
        .verify_credentials(&user.username, "TestWorkflow123!")
        .await
        .expect("Failed to verify credentials");
    assert!(login_check.is_some());
    
    let verified_user = login_check.unwrap();
    assert_eq!(verified_user.user_id, user.user_id);
    
    // Test 3: Verify token is valid
    let claims = setup.auth_service.verify_token(&user.auth_token)
        .expect("Failed to verify token");
    assert_eq!(claims.sub, user.user_id.to_string());
    
    // Cleanup
    setup.cleanup_test_data().await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_invalid_login_attempts() {
    let mut setup = TestSetup::new().await;
    let user = setup.create_test_user("login_test_user").await;
    
    // Test 1: Wrong password
    let invalid_login = setup.user_service
        .verify_credentials(&user.username, "WrongPassword!")
        .await
        .expect("Credential check should not fail");
    assert!(invalid_login.is_none());
    
    // Test 2: Non-existent user
    let no_user = setup.user_service
        .verify_credentials("nonexistent_user", "TestWorkflow123!")
        .await
        .expect("Credential check should not fail");
    assert!(no_user.is_none());
    
    // Test 3: Empty credentials
    let empty_user = setup.user_service
        .verify_credentials("", "TestWorkflow123!")
        .await;
    assert!(empty_user.is_err() || empty_user.unwrap().is_none());
    
    setup.cleanup_test_data().await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_token_validation_workflow() {
    let mut setup = TestSetup::new().await;
    let user = setup.create_test_user("token_test_user").await;
    
    // Test 1: Valid token
    let claims = setup.auth_service.verify_token(&user.auth_token)
        .expect("Valid token should verify");
    assert_eq!(claims.sub, user.user_id.to_string());
    
    // Test 2: Invalid token
    assert!(setup.auth_service.verify_token("invalid_token").is_err());
    
    // Test 3: Empty token
    assert!(setup.auth_service.verify_token("").is_err());
    
    // Test 4: Malformed token
    assert!(setup.auth_service.verify_token("not.a.jwt.token").is_err());
    
    setup.cleanup_test_data().await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_multiple_users_workflow() {
    let mut setup = TestSetup::new().await;
    
    // Create multiple test users
    let user_types = vec!["admin_user", "analyst_user", "regular_user", "readonly_user"];
    let mut created_users = Vec::new();
    
    for user_type in &user_types {
        let user = setup.create_test_user(user_type).await;
        created_users.push(user.clone());
    }
    
    // Test 1: All users were created successfully
    assert_eq!(created_users.len(), 4);
    
    // Test 2: Each user can log in with their credentials
    for user in &created_users {
        let login_check = setup.user_service
            .verify_credentials(&user.username, "TestWorkflow123!")
            .await
            .expect("Failed to verify credentials");
        assert!(login_check.is_some());
        
        let verified_user = login_check.unwrap();
        assert_eq!(verified_user.user_id, user.user_id);
    }
    
    // Test 3: Each user has a valid token
    for user in &created_users {
        let claims = setup.auth_service.verify_token(&user.auth_token)
            .expect("Failed to verify token");
        assert_eq!(claims.sub, user.user_id.to_string());
    }
    
    // Test 4: Users are distinct
    let mut user_ids = created_users.iter().map(|u| u.user_id).collect::<Vec<_>>();
    user_ids.sort();
    user_ids.dedup();
    assert_eq!(user_ids.len(), 4); // No duplicates
    
    setup.cleanup_test_data().await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_password_security_workflow() {
    let mut setup = TestSetup::new().await;
    
    // Test 1: Strong passwords are accepted
    let strong_password_user = CreateUserRequest {
        username: "strong_pass_user".to_string(),
        email: "strong@workflow-test.com".to_string(),
        password: "SuperSecure123!@#".to_string(),
    };
    
    let user1 = setup.user_service.create_user(strong_password_user).await;
    assert!(user1.is_ok());
    
    // Test 2: Verify password hashing
    if let Ok(user) = user1 {
        let login_check = setup.user_service
            .verify_credentials(&user.username, "SuperSecure123!@#")
            .await
            .expect("Failed to verify credentials");
        assert!(login_check.is_some());
        
        // Wrong password should fail
        let wrong_pass = setup.user_service
            .verify_credentials(&user.username, "WrongPassword")
            .await
            .expect("Credential check should not fail");
        assert!(wrong_pass.is_none());
    }
    
    setup.cleanup_test_data().await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_user_data_integrity() {
    let mut setup = TestSetup::new().await;
    let user = setup.create_test_user("integrity_user").await;
    
    // Test 1: User data is correctly stored and retrieved
    let login_user = setup.user_service
        .verify_credentials(&user.username, "TestWorkflow123!")
        .await
        .expect("Failed to verify credentials")
        .expect("User should exist");
    
    assert_eq!(login_user.user_id, user.user_id);
    assert_eq!(login_user.username, user.username);
    assert_eq!(login_user.email, user.email);
    assert!(login_user.is_active);
    
    // Test 2: Created and updated timestamps exist
    assert!(login_user.created_at <= login_user.updated_at);
    
    setup.cleanup_test_data().await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_concurrent_user_operations() {
    let setup = TestSetup::new().await;
    
    // Test concurrent user creation (simulating multiple registration attempts)
    let usernames = vec!["concurrent1", "concurrent2", "concurrent3", "concurrent4"];
    let mut handles = vec![];
    
    for username in usernames {
        let user_service_pool = setup.database.pool().clone();
        let auth_service = setup.auth_service.clone();
        
        let handle = tokio::spawn(async move {
            let user_service = UserService::new(user_service_pool, auth_service);
            let create_request = CreateUserRequest {
                username: username.to_string(),
                email: format!("{}@workflow-test.com", username),
                password: "ConcurrentTest123!".to_string(),
            };
            
            let result = user_service.create_user(create_request).await;
            (username, result)
        });
        
        handles.push(handle);
    }
    
    // Wait for all concurrent operations to complete
    let mut success_count = 0;
    for handle in handles {
        let (username, result) = handle.await.expect("Task should complete");
        if result.is_ok() {
            success_count += 1;
            println!("Successfully created user: {}", username);
        } else {
            println!("Failed to create user {}: {:?}", username, result.err());
        }
    }
    
    // All users should be created successfully
    assert_eq!(success_count, 4);
    
    setup.cleanup_test_data().await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_user_session_workflow() {
    let mut setup = TestSetup::new().await;
    let user = setup.create_test_user("session_user").await;
    
    // Test 1: User can authenticate and receive token
    let login_result = setup.user_service
        .verify_credentials(&user.username, "TestWorkflow123!")
        .await
        .expect("Login should work")
        .expect("User should exist");
    
    let token = setup.auth_service.generate_token(login_result.user_id)
        .expect("Token generation should work");
    
    // Test 2: Token can be validated multiple times
    for _ in 0..5 {
        let claims = setup.auth_service.verify_token(&token)
            .expect("Token should remain valid");
        assert_eq!(claims.sub, user.user_id.to_string());
    }
    
    // Test 3: Different tokens for same user should be valid
    let token2 = setup.auth_service.generate_token(login_result.user_id)
        .expect("Second token generation should work");
    
    let claims1 = setup.auth_service.verify_token(&token).expect("First token valid");
    let claims2 = setup.auth_service.verify_token(&token2).expect("Second token valid");
    
    assert_eq!(claims1.sub, claims2.sub);
    assert_eq!(claims1.sub, user.user_id.to_string());
    
    setup.cleanup_test_data().await.expect("Failed to cleanup");
}

#[tokio::test] 
async fn test_error_handling_workflow() {
    let mut setup = TestSetup::new().await;
    
    // Test 1: Duplicate username should fail
    let user1 = CreateUserRequest {
        username: "duplicate_user".to_string(),
        email: "user1@workflow-test.com".to_string(),
        password: "Password123!".to_string(),
    };
    
    let user2 = CreateUserRequest {
        username: "duplicate_user".to_string(), // Same username
        email: "user2@workflow-test.com".to_string(),
        password: "Password123!".to_string(),
    };
    
    let first_result = setup.user_service.create_user(user1).await;
    assert!(first_result.is_ok());
    
    let second_result = setup.user_service.create_user(user2).await;
    assert!(second_result.is_err()); // Should fail due to duplicate username
    
    // Test 2: Duplicate email should fail
    let user3 = CreateUserRequest {
        username: "different_user".to_string(),
        email: "user1@workflow-test.com".to_string(), // Same email as user1
        password: "Password123!".to_string(),
    };
    
    let third_result = setup.user_service.create_user(user3).await;
    assert!(third_result.is_err()); // Should fail due to duplicate email
    
    setup.cleanup_test_data().await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_database_connection_workflow() {
    // Test that we can establish database connection
    let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
        "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
    });
    
    let database = Database::new(&database_url).await;
    assert!(database.is_ok());
    
    // Test basic query
    let db = database.unwrap();
    let result = sqlx::query!("SELECT 1 as test_value")
        .fetch_one(db.pool())
        .await;
    
    assert!(result.is_ok());
    let row = result.unwrap();
    assert_eq!(row.test_value.unwrap(), 1);
}

// Integration test covering the complete user workflow
#[tokio::test]
async fn test_complete_user_workflow_integration() {
    let mut setup = TestSetup::new().await;
    
    println!("🔄 Starting complete user workflow test...");
    
    // Step 1: User Registration
    println!("📝 Step 1: User Registration");
    let user = setup.create_test_user("integration_user").await;
    assert!(!user.user_id.to_string().is_empty());
    println!("✅ User created: {} ({})", user.username, user.user_id);
    
    // Step 2: User Authentication
    println!("🔐 Step 2: User Authentication");
    let login_result = setup.user_service
        .verify_credentials(&user.username, "TestWorkflow123!")
        .await
        .expect("Login should work")
        .expect("User should exist");
    println!("✅ Login successful for: {}", login_result.username);
    
    // Step 3: Token Generation and Validation
    println!("🎫 Step 3: Token Operations");
    let claims = setup.auth_service.verify_token(&user.auth_token)
        .expect("Token should be valid");
    assert_eq!(claims.sub, user.user_id.to_string());
    println!("✅ Token validated for user: {}", claims.sub);
    
    // Step 4: Multiple Session Support
    println!("🔄 Step 4: Multiple Sessions");
    let second_token = setup.auth_service.generate_token(user.user_id)
        .expect("Second token generation should work");
    let second_claims = setup.auth_service.verify_token(&second_token)
        .expect("Second token should be valid");
    assert_eq!(second_claims.sub, user.user_id.to_string());
    println!("✅ Multiple sessions supported");
    
    // Step 5: Security Validation
    println!("🛡️  Step 5: Security Checks");
    // Wrong password fails
    let wrong_login = setup.user_service
        .verify_credentials(&user.username, "WrongPassword")
        .await
        .expect("Should not error")
        .is_none();
    assert!(wrong_login);
    
    // Invalid token fails  
    assert!(setup.auth_service.verify_token("invalid.token.here").is_err());
    println!("✅ Security validations passed");
    
    // Step 6: Data Integrity
    println!("📊 Step 6: Data Integrity");
    assert_eq!(login_result.user_id, user.user_id);
    assert_eq!(login_result.username, user.username);
    assert_eq!(login_result.email, user.email);
    assert!(login_result.is_active);
    println!("✅ Data integrity verified");
    
    // Step 7: Cleanup
    println!("🧹 Step 7: Cleanup");
    setup.cleanup_test_data().await.expect("Cleanup should work");
    println!("✅ Test data cleaned up");
    
    println!("🎉 Complete user workflow test PASSED!");
}