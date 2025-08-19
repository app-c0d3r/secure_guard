use secureguard_api::database::Database;
use secureguard_api::services::{
    auth_service::AuthService, 
    user_service::UserService,
};
use secureguard_api::middleware::rbac::{UserPermissions, get_user_permissions};
use secureguard_shared::{CreateUserRequest, LoginRequest, UserRole};
use uuid::Uuid;
use std::collections::HashMap;
use sqlx::PgPool;
use std::collections::HashSet;

// Test helper struct for managing test users with roles
#[derive(Debug, Clone)]
pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
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
    
    pub async fn create_test_user(&mut self, username: &str, role: UserRole) -> &TestUser {
        let unique_id = Uuid::new_v4().to_string()[..8].to_string();
        let unique_username = format!("{}_{}", username, unique_id);
        let email = format!("{}@test.com", unique_username);
        let password = "TestPassword123!";
        
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
            
        // Assign role via database directly (simulating admin assignment)
        self.assign_role_to_user(user.user_id, role.clone()).await;
        
        let test_user = TestUser {
            user_id: user.user_id,
            username: unique_username,
            email,
            role: role.clone(),
            auth_token,
        };
        
        self.test_users.insert(username.to_string(), test_user);
        self.test_users.get(username).unwrap()
    }
    
    async fn assign_role_to_user(&self, user_id: Uuid, role: UserRole) {
        // This would typically be done through the RoleManagementService
        // For testing, we'll directly insert into the database
        let _ = sqlx::query!(
            r#"
            INSERT INTO rbac.user_roles (user_role_id, user_id, role_id, assigned_by, is_active)
            SELECT gen_random_uuid(), $1, role_id, $1, TRUE
            FROM rbac.roles 
            WHERE role_slug = $2
            ON CONFLICT DO NOTHING
            "#,
            user_id,
            role.to_slug()
        )
        .execute(self.database.pool())
        .await;
    }
}

// SECURITY ANALYST WORKFLOW TESTS
#[tokio::test]
async fn test_security_analyst_full_workflow() {
    let mut setup = TestSetup::new().await;
    
    // Create security analyst user
    let analyst = setup.create_test_user("sec_analyst", UserRole::SecurityAnalyst).await;
    
    // Test 1: Login and authentication
    let credentials_check = setup.user_service
        .verify_credentials(&analyst.username, "TestPassword123!")
        .await
        .expect("Failed to verify analyst credentials");
    assert!(credentials_check.is_some());
    
    // Test 2: Check role permissions
    let permissions = get_user_permissions(setup.database.pool(), analyst.user_id)
        .await
        .expect("Failed to get analyst permissions");
    
    assert_eq!(permissions.highest_role.to_slug(), "security_analyst");
    assert!(permissions.has_permission("security.monitoring"));
    assert!(permissions.has_permission("security.incidents"));
    assert!(permissions.has_permission("agents.read"));
    
    // Test 3: Security monitoring access
    assert!(permissions.can_access_resource("security", "monitoring"));
    assert!(permissions.can_access_resource("security", "incidents"));
    assert!(permissions.can_access_resource("security", "response"));
    
    // Test 4: Agent monitoring capabilities
    assert!(permissions.can_access_resource("agents", "read"));
    assert!(permissions.can_access_resource("agents", "update")); // For status updates
    
    // Test 5: Cannot access admin functions
    assert!(!permissions.has_permission("users.create"));
    assert!(!permissions.has_permission("system.admin"));
    assert!(!permissions.can_manage_users);
    assert!(!permissions.can_admin_system);
    
    // Test 6: Can access necessary secrets for security work
    assert!(permissions.can_access_secrets || permissions.has_permission("secrets.read"));
}

#[tokio::test] 
async fn test_analyst_daily_tasks_workflow() {
    let mut setup = TestSetup::new().await;
    let analyst = setup.create_test_user("analyst_daily", UserRole::SecurityAnalyst).await;
    
    let permissions = get_user_permissions(setup.database.pool(), analyst.user_id)
        .await
        .expect("Failed to get permissions");
    
    // Daily Task 1: Review security incidents
    assert!(permissions.has_permission("security.incidents"));
    
    // Daily Task 2: Monitor agent health
    assert!(permissions.has_permission("agents.read"));
    
    // Daily Task 3: Analyze threat data
    assert!(permissions.has_permission("security.monitoring"));
    
    // Daily Task 4: Generate reports
    assert!(permissions.has_permission("audit.read"));
    
    // Daily Task 5: Respond to alerts
    assert!(permissions.has_permission("security.response"));
    
    // Verify analyst can't perform admin tasks
    assert!(!permissions.has_permission("users.delete"));
    assert!(!permissions.has_permission("system.config"));
}

// ADMIN ROLE WORKFLOW TESTS
#[tokio::test]
async fn test_admin_user_management_workflow() {
    let mut setup = TestSetup::new().await;
    let admin = setup.create_test_user("admin_user", UserRole::Admin).await;
    
    let permissions = get_user_permissions(setup.database.pool(), admin.user_id)
        .await
        .expect("Failed to get admin permissions");
    
    // Admin can manage users
    assert!(permissions.can_manage_users);
    assert!(permissions.has_permission("users.create"));
    assert!(permissions.has_permission("users.read"));
    assert!(permissions.has_permission("users.update"));
    assert!(permissions.has_permission("users.delete"));
    assert!(permissions.has_permission("users.roles"));
    
    // Admin can manage agents
    assert!(permissions.has_permission("agents.create"));
    assert!(permissions.has_permission("agents.update"));
    assert!(permissions.has_permission("agents.delete"));
    
    // Admin cannot access system-level functions (reserved for SystemAdmin)
    assert!(!permissions.can_admin_system);
    assert!(!permissions.has_permission("system.admin"));
}

// SYSTEM ADMIN WORKFLOW TESTS
#[tokio::test]
async fn test_system_admin_full_access_workflow() {
    let mut setup = TestSetup::new().await;
    let sys_admin = setup.create_test_user("sys_admin", UserRole::SystemAdmin).await;
    
    let permissions = get_user_permissions(setup.database.pool(), sys_admin.user_id)
        .await
        .expect("Failed to get system admin permissions");
    
    // System admin has all access
    assert!(permissions.can_admin_system);
    assert!(permissions.can_manage_users);
    assert!(permissions.can_access_secrets);
    
    // All key permissions
    assert!(permissions.has_permission("system.admin"));
    assert!(permissions.has_permission("system.config"));
    assert!(permissions.has_permission("system.maintenance"));
    assert!(permissions.has_permission("users.create"));
    assert!(permissions.has_permission("users.delete"));
    assert!(permissions.has_permission("secrets.create"));
    assert!(permissions.has_permission("secrets.delete"));
}

// USER ROLE WORKFLOW TESTS
#[tokio::test]
async fn test_regular_user_limited_workflow() {
    let mut setup = TestSetup::new().await;
    let user = setup.create_test_user("regular_user", UserRole::User).await;
    
    let permissions = get_user_permissions(setup.database.pool(), user.user_id)
        .await
        .expect("Failed to get user permissions");
    
    // Basic user permissions
    assert!(permissions.has_permission("api.read"));
    assert!(!permissions.can_manage_users);
    assert!(!permissions.can_admin_system);
    assert!(!permissions.can_access_secrets);
    
    // Cannot access admin functions
    assert!(!permissions.has_permission("users.create"));
    assert!(!permissions.has_permission("users.delete"));
    assert!(!permissions.has_permission("system.admin"));
    assert!(!permissions.has_permission("secrets.read"));
}

// READONLY ROLE WORKFLOW TESTS
#[tokio::test]
async fn test_readonly_user_view_only_workflow() {
    let mut setup = TestSetup::new().await;
    let readonly = setup.create_test_user("readonly_user", UserRole::ReadOnly).await;
    
    let permissions = get_user_permissions(setup.database.pool(), readonly.user_id)
        .await
        .expect("Failed to get readonly permissions");
    
    // Should have read permissions only
    assert!(permissions.has_permission("agents.read") || permissions.has_permission("api.read"));
    
    // Should NOT have write permissions
    assert!(!permissions.has_permission("agents.create"));
    assert!(!permissions.has_permission("agents.update"));
    assert!(!permissions.has_permission("agents.delete"));
    assert!(!permissions.has_permission("users.create"));
    assert!(!permissions.can_manage_users);
}

// CROSS-ROLE INTERACTION TESTS
#[tokio::test]
async fn test_role_hierarchy_enforcement() {
    let mut setup = TestSetup::new().await;
    
    // Create users with different roles
    let sys_admin = setup.create_test_user("hierarchy_sysadmin", UserRole::SystemAdmin).await;
    let admin = setup.create_test_user("hierarchy_admin", UserRole::Admin).await;
    let analyst = setup.create_test_user("hierarchy_analyst", UserRole::SecurityAnalyst).await;
    let user = setup.create_test_user("hierarchy_user", UserRole::User).await;
    
    // Test hierarchy levels
    assert!(UserRole::SystemAdmin.hierarchy_level() > UserRole::Admin.hierarchy_level());
    assert!(UserRole::Admin.hierarchy_level() > UserRole::SecurityAnalyst.hierarchy_level());
    assert!(UserRole::SecurityAnalyst.hierarchy_level() > UserRole::User.hierarchy_level());
    
    // Verify permissions reflect hierarchy
    let sys_admin_perms = get_user_permissions(setup.database.pool(), sys_admin.user_id).await.unwrap();
    let admin_perms = get_user_permissions(setup.database.pool(), admin.user_id).await.unwrap();
    let analyst_perms = get_user_permissions(setup.database.pool(), analyst.user_id).await.unwrap();
    let user_perms = get_user_permissions(setup.database.pool(), user.user_id).await.unwrap();
    
    // SystemAdmin has most permissions
    assert!(sys_admin_perms.permissions.len() >= admin_perms.permissions.len());
    assert!(admin_perms.permissions.len() >= analyst_perms.permissions.len());
    assert!(analyst_perms.permissions.len() >= user_perms.permissions.len());
}

// AUTHENTICATION AND SESSION TESTS
#[tokio::test]
async fn test_multi_role_authentication_workflow() {
    let mut setup = TestSetup::new().await;
    
    // Create multiple users with different roles
    let users = vec![
        ("auth_admin", UserRole::Admin),
        ("auth_analyst", UserRole::SecurityAnalyst),
        ("auth_user", UserRole::User),
        ("auth_readonly", UserRole::ReadOnly),
    ];
    
    for (username, role) in users {
        let user = setup.create_test_user(username, role.clone()).await;
        
        // Test login
        let login_result = setup.user_service
            .verify_credentials(&user.username, "TestPassword123!")
            .await
            .expect("Failed to verify credentials");
        assert!(login_result.is_some());
        
        // Test token validation
        let claims = setup.auth_service.verify_token(&user.auth_token)
            .expect("Failed to verify token");
        assert_eq!(claims.sub, user.user_id.to_string());
        
        // Verify role is correctly assigned
        let permissions = get_user_permissions(setup.database.pool(), user.user_id)
            .await
            .expect("Failed to get permissions");
        assert_eq!(permissions.highest_role.to_slug(), role.to_slug());
    }
}

// COMPREHENSIVE END-TO-END WORKFLOW TEST
#[tokio::test]
async fn test_complete_application_workflow() {
    let mut setup = TestSetup::new().await;
    
    // Create a realistic scenario with multiple users
    let sys_admin = setup.create_test_user("e2e_sysadmin", UserRole::SystemAdmin).await;
    let security_team_lead = setup.create_test_user("e2e_sec_lead", UserRole::SecurityAnalyst).await;
    let admin_user = setup.create_test_user("e2e_admin", UserRole::Admin).await;
    let regular_user = setup.create_test_user("e2e_user", UserRole::User).await;
    
    // Scenario 1: System Admin sets up the system
    let sys_perms = get_user_permissions(setup.database.pool(), sys_admin.user_id).await.unwrap();
    assert!(sys_perms.can_admin_system);
    assert!(sys_perms.has_permission("system.config"));
    
    // Scenario 2: Admin creates regular users and assigns roles
    let admin_perms = get_user_permissions(setup.database.pool(), admin_user.user_id).await.unwrap();
    assert!(admin_perms.can_manage_users);
    assert!(admin_perms.has_permission("users.create"));
    
    // Scenario 3: Security Analyst monitors threats and agents
    let sec_perms = get_user_permissions(setup.database.pool(), security_team_lead.user_id).await.unwrap();
    assert!(sec_perms.has_permission("security.monitoring"));
    assert!(sec_perms.has_permission("agents.read"));
    assert!(sec_perms.has_permission("security.incidents"));
    
    // Scenario 4: Regular user has limited access
    let user_perms = get_user_permissions(setup.database.pool(), regular_user.user_id).await.unwrap();
    assert!(!user_perms.can_manage_users);
    assert!(!user_perms.can_admin_system);
    assert!(user_perms.has_permission("api.read"));
    
    // Scenario 5: Verify cross-role boundaries are maintained
    assert!(sys_perms.permissions.len() > admin_perms.permissions.len());
    assert!(admin_perms.permissions.len() > sec_perms.permissions.len());
    assert!(sec_perms.permissions.len() > user_perms.permissions.len());
}

// TEST ERROR SCENARIOS AND EDGE CASES
#[tokio::test]
async fn test_invalid_credentials_and_permissions() {
    let mut setup = TestSetup::new().await;
    let user = setup.create_test_user("invalid_test", UserRole::User).await;
    
    // Test invalid password
    let invalid_login = setup.user_service
        .verify_credentials(&user.username, "WrongPassword")
        .await
        .expect("Credential check should not fail");
    assert!(invalid_login.is_none());
    
    // Test invalid token
    assert!(setup.auth_service.verify_token("invalid_token").is_err());
    
    // Test permissions for non-existent user
    let fake_user_id = Uuid::new_v4();
    let result = get_user_permissions(setup.database.pool(), fake_user_id).await;
    // Should return default guest permissions or error
    if let Ok(perms) = result {
        assert_eq!(perms.highest_role.to_slug(), "guest");
        assert!(perms.permissions.is_empty());
    }
}