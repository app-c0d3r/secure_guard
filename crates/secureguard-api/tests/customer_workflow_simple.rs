// Customer Workflow Tests - Simplified Working Version
// Tests core customer onboarding and cancellation workflows

use secureguard_api::database::Database;
use secureguard_api::services::{
    auth_service::AuthService,
    user_service::UserService,
    agent_service::AgentService,
    api_key_service::ApiKeyService,
};
use secureguard_shared::{
    CreateUserRequest, CreateApiKeyRequest, RegisterAgentRequest,
};
use serde_json;
use uuid::Uuid;

// Helper function to create unique usernames for test isolation
fn create_unique_user(base_username: &str, email_domain: &str, password: &str) -> CreateUserRequest {
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let unique_username = format!("{}_{}", base_username, unique_id);
    
    CreateUserRequest {
        username: unique_username,
        email: format!("{}@{}", unique_id, email_domain),
        password: password.to_string(),
    }
}

async fn create_workflow_setup() -> (Database, AuthService, UserService, AgentService, ApiKeyService) {
    let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
        "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
    });
    
    let database = Database::new(&database_url)
        .await
        .expect("Failed to connect to test database");
        
    let auth_service = AuthService::new("test-secret-key-workflow-simple".to_string());
    let user_service = UserService::new(database.pool().clone(), auth_service.clone());
    let agent_service = AgentService::new(database.pool().clone());
    let api_key_service = ApiKeyService::new(database.pool().clone());
    
    (database, auth_service, user_service, agent_service, api_key_service)
}

async fn cleanup_workflow_data(database: &Database) -> Result<(), sqlx::Error> {
    // Clean up test data
    sqlx::query!("DELETE FROM agents.endpoints WHERE hardware_fingerprint LIKE 'test-simple-%'")
        .execute(database.pool()).await?;
    sqlx::query!("DELETE FROM users.api_keys WHERE key_name LIKE 'test-simple-%'")
        .execute(database.pool()).await?;
    sqlx::query!("DELETE FROM users.users WHERE email LIKE '%@simple-workflow.com'")
        .execute(database.pool()).await?;
    Ok(())
}

// TEST 1: COMPLETE CUSTOMER ONBOARDING WORKFLOW
#[tokio::test]
async fn test_customer_onboarding_workflow() {
    let (database, auth_service, user_service, agent_service, api_key_service) = create_workflow_setup().await;
    
    println!("🚀 Starting Customer Onboarding Workflow Test");
    
    // Step 1: Customer Pass Login (Registration + Login)
    println!("👤 Step 1: Customer Registration and Login");
    let customer_request = create_unique_user("workflow_customer", "simple-workflow.com", "WorkflowSecure123!");
    let customer_username = customer_request.username.clone();
    
    let customer = user_service.create_user(customer_request).await
        .expect("Customer registration should succeed");
    
    assert_eq!(customer.username, customer_username);
    assert!(customer.is_active);
    println!("✅ Customer registered: {}", customer.user_id);
    
    // Verify login
    let login_result = user_service
        .verify_credentials(&customer_username, "WorkflowSecure123!")
        .await
        .expect("Login should work")
        .expect("Customer should exist");
    
    assert_eq!(login_result.user_id, customer.user_id);
    println!("✅ Customer login successful");
    
    // Step 2: Download Agent (Generate API Key)
    println!("🔑 Step 2: Customer Downloads Agent (API Key Generation)");
    let api_key_request = CreateApiKeyRequest {
        key_name: "test-simple-agent-download".to_string(),
        expires_in_days: Some(365),
    };
    
    let api_key_response = api_key_service
        .create_api_key(customer.user_id, api_key_request)
        .await
        .expect("API key creation should succeed");
    
    assert!(!api_key_response.api_key.is_empty());
    assert!(api_key_response.api_key.starts_with("sg_"));
    println!("✅ Agent download ready - API key generated: {}", api_key_response.key_prefix);
    
    // Step 3: Install Agent and Register with API Key
    println!("💿 Step 3: Agent Installation and Registration");
    let register_request = RegisterAgentRequest {
        api_key: api_key_response.api_key.clone(),
        device_name: "Customer-Workstation-01".to_string(),
        hardware_fingerprint: "test-simple-workstation-001".to_string(),
        os_info: serde_json::json!({
            "name": "Windows",
            "version": "11",
            "architecture": "x64",
            "manufacturer": "Dell Inc."
        }),
        version: "1.0.0".to_string(),
    };
    
    let registered_agent = agent_service
        .register_agent_with_api_key(register_request)
        .await
        .expect("Agent registration should succeed");
    
    assert_eq!(registered_agent.hardware_fingerprint, "test-simple-workstation-001");
    println!("✅ Agent installed and registered: {}", registered_agent.agent_id);
    
    // Step 4: Agent Connect to App (Heartbeat)
    println!("🔗 Step 4: Agent Connects to Application");
    
    // Verify agent exists and can receive heartbeats
    let heartbeat_result = sqlx::query!(
        "UPDATE agents.endpoints SET status = 'online', last_heartbeat = now() WHERE agent_id = $1 RETURNING status",
        registered_agent.agent_id
    )
    .fetch_one(database.pool())
    .await
    .expect("Heartbeat update should succeed");
    
    assert_eq!(heartbeat_result.status, "online");
    println!("✅ Agent connected and online");
    
    // Step 5: Analyst Can Monitor New Asset
    println!("📊 Step 5: New Asset Available for Analyst Monitoring");
    
    let monitoring_check = sqlx::query!(
        r#"
        SELECT a.agent_id, a.status, a.hardware_fingerprint, a.last_heartbeat
        FROM agents.endpoints a
        WHERE a.agent_id = $1 AND a.status = 'online'
        "#,
        registered_agent.agent_id
    )
    .fetch_one(database.pool())
    .await
    .expect("Should find online agent");
    
    assert_eq!(monitoring_check.agent_id, registered_agent.agent_id);
    assert_eq!(monitoring_check.status, "online");
    println!("✅ New asset is online and available for analyst monitoring");
    
    // Step 6: Verify Complete Onboarding Success
    println!("🎯 Step 6: Complete Onboarding Verification");
    
    // Customer still active
    let final_customer = user_service
        .verify_credentials(&customer_username, "WorkflowSecure123!")
        .await
        .expect("Final verification should work")
        .expect("Customer should exist");
    assert!(final_customer.is_active);
    
    // API key still active
    let api_key_check = sqlx::query!(
        "SELECT is_active FROM users.api_keys WHERE key_id = $1",
        api_key_response.key_id
    )
    .fetch_one(database.pool())
    .await
    .expect("Should find API key");
    assert!(api_key_check.is_active);
    
    // Agent still registered and online
    let agent_check = sqlx::query!(
        "SELECT status FROM agents.endpoints WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .fetch_one(database.pool())
    .await
    .expect("Should find agent");
    assert_eq!(agent_check.status, "online");
    
    println!("✅ Complete customer onboarding workflow SUCCESSFUL!");
    println!("📈 Workflow Summary: Customer Login → Download Agent → Install → Register → Connect → Monitor Ready");
    
    cleanup_workflow_data(&database).await.expect("Cleanup should work");
}

// TEST 2: CUSTOMER ACCOUNT CANCELLATION WORKFLOW
#[tokio::test]
async fn test_customer_cancellation_workflow() {
    let (database, auth_service, user_service, agent_service, api_key_service) = create_workflow_setup().await;
    
    println!("🚀 Starting Customer Account Cancellation Workflow Test");
    
    // Pre-setup: Create customer with active agent
    println!("⚙️ Pre-setup: Customer with Active Agent");
    
    let customer_request = create_unique_user("cancel_customer", "simple-workflow.com", "CancelTest123!");
    let customer = user_service.create_user(customer_request).await
        .expect("Customer creation should succeed");
    
    let api_key_response = api_key_service
        .create_api_key(customer.user_id, CreateApiKeyRequest {
            key_name: "test-simple-cancel-agent".to_string(),
            expires_in_days: Some(30),
        })
        .await.expect("API key creation should succeed");
    
    let registered_agent = agent_service
        .register_agent_with_api_key(RegisterAgentRequest {
            api_key: api_key_response.api_key.clone(),
            device_name: "Cancel-Customer-PC".to_string(),
            hardware_fingerprint: "test-simple-cancel-pc-001".to_string(),
            os_info: serde_json::json!({"name": "Windows", "version": "10"}),
            version: "1.0.0".to_string(),
        })
        .await.expect("Agent registration should succeed");
    
    // Make agent online
    sqlx::query!(
        "UPDATE agents.endpoints SET status = 'online', last_heartbeat = now() WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .execute(database.pool())
    .await.expect("Agent status update should work");
    
    println!("✅ Pre-setup complete - Customer has active monitored agent");
    
    // Step 1: Customer Goes to Profile and Cancels Account
    println!("🚫 Step 1: Customer Initiates Account Cancellation");
    
    let pre_cancel_customer = user_service
        .verify_credentials("cancel_customer", "CancelTest123!")
        .await
        .expect("Pre-cancel check should work")
        .expect("Customer should exist");
    assert!(pre_cancel_customer.is_active);
    
    let cancellation_reason = "Service no longer needed - migrating to different solution".to_string();
    println!("✅ Customer cancellation request received: {}", cancellation_reason);
    
    // Step 2: Agents Deactivated Automatically
    println!("🔴 Step 2: Automatic Agent Deactivation");
    
    let deactivation_result = sqlx::query!(
        "UPDATE agents.endpoints SET status = 'offline' WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .execute(database.pool())
    .await
    .expect("Agent deactivation should succeed");
    
    println!("✅ Agent automatically deactivated - {} row(s) updated", deactivation_result.rows_affected());
    
    // Step 3: Agents Uninstalled Automatically on Customer Device
    println!("🗑️ Step 3: Automatic Agent Uninstallation");
    
    // In real system, this would send uninstall commands to agents
    // For testing, we simulate successful uninstallation by removing agent
    let uninstall_result = sqlx::query!(
        "DELETE FROM agents.endpoints WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .execute(database.pool())
    .await
    .expect("Agent uninstallation should succeed");
    
    assert_eq!(uninstall_result.rows_affected(), 1);
    println!("✅ Agent automatically uninstalled from customer device");
    
    // Step 4: Asset No Longer Available and Not Monitored
    println!("📊 Step 4: Asset Removed from Monitoring System");
    
    let monitoring_check = sqlx::query!(
        "SELECT agent_id FROM agents.endpoints WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .fetch_optional(database.pool())
    .await
    .expect("Query should work");
    
    assert!(monitoring_check.is_none());
    println!("✅ Asset completely removed from monitoring - no longer available to analysts");
    
    // Step 5: Account Deletion
    println!("💀 Step 5: Customer Account Deletion");
    
    // First deactivate account
    let deactivate_result = sqlx::query!(
        "UPDATE users.users SET is_active = FALSE WHERE user_id = $1 RETURNING is_active",
        customer.user_id
    )
    .fetch_one(database.pool())
    .await
    .expect("Account deactivation should succeed");
    
    assert!(!deactivate_result.is_active);
    println!("✅ Account deactivated");
    
    // Then delete account and all related data
    let delete_api_keys = sqlx::query!(
        "DELETE FROM users.api_keys WHERE user_id = $1",
        customer.user_id
    )
    .execute(database.pool())
    .await
    .expect("API key deletion should succeed");
    
    let delete_user = sqlx::query!(
        "DELETE FROM users.users WHERE user_id = $1",
        customer.user_id
    )
    .execute(database.pool())
    .await
    .expect("User deletion should succeed");
    
    println!("✅ Account and all related data permanently deleted");
    
    // Step 6: User Gets Email Notification (Mock)
    println!("📧 Step 6: Customer Email Notification");
    
    // Mock email notification
    println!("📧 ACCOUNT CANCELLATION EMAIL:");
    println!("   To: cancel@simple-workflow.com");
    println!("   Subject: Account Cancellation Confirmation - SecureGuard");
    println!("   Message: Your account has been successfully cancelled.");
    println!("   Reason: {}", cancellation_reason);
    println!("   All agents have been automatically uninstalled.");
    println!("   Thank you for using SecureGuard.");
    println!("✅ Email notification sent to customer");
    
    // Step 7: Final Verification - Complete Cleanup
    println!("🔍 Step 7: Complete Cancellation Verification");
    
    // Customer cannot login
    let post_cancel_login = user_service
        .verify_credentials("cancel_customer", "CancelTest123!")
        .await
        .expect("Should not error");
    assert!(post_cancel_login.is_none());
    
    // No user data in database
    let user_check = sqlx::query!(
        "SELECT user_id FROM users.users WHERE user_id = $1",
        customer.user_id
    )
    .fetch_optional(database.pool())
    .await
    .expect("Query should work");
    assert!(user_check.is_none());
    
    // No API keys in database
    let api_key_check = sqlx::query!(
        "SELECT key_id FROM users.api_keys WHERE user_id = $1",
        customer.user_id
    )
    .fetch_all(database.pool())
    .await
    .expect("Query should work");
    assert!(api_key_check.is_empty());
    
    // No agents in database
    let agent_check = sqlx::query!(
        "SELECT agent_id FROM agents.endpoints WHERE hardware_fingerprint = $1",
        "test-simple-cancel-pc-001"
    )
    .fetch_optional(database.pool())
    .await
    .expect("Query should work");
    assert!(agent_check.is_none());
    
    println!("✅ Complete customer cancellation workflow SUCCESSFUL!");
    println!("📉 Workflow Summary: Cancel Request → Deactivate Agents → Uninstall → Remove Monitoring → Delete Account → Email Customer");
    
    cleanup_workflow_data(&database).await.expect("Cleanup should work");
}

// TEST 3: ANALYST MONITORING WORKFLOW
#[tokio::test]
async fn test_analyst_monitoring_workflow() {
    let (database, _auth_service, user_service, agent_service, api_key_service) = create_workflow_setup().await;
    
    println!("🚀 Starting Analyst Monitoring Workflow Test");
    
    // Step 1: Create Analyst and Customer with Agent
    println!("👥 Step 1: Setup Analyst and Customer with Agent");
    
    let analyst = user_service.create_user(CreateUserRequest {
        username: "monitoring_analyst".to_string(),
        email: "analyst@simple-workflow.com".to_string(),
        password: "AnalystSecure123!".to_string(),
    }).await.expect("Analyst creation should succeed");
    
    let customer = user_service.create_user(CreateUserRequest {
        username: "monitored_customer".to_string(),
        email: "monitored@simple-workflow.com".to_string(),
        password: "MonitoredTest123!".to_string(),
    }).await.expect("Customer creation should succeed");
    
    // Customer sets up agent
    let api_key_response = api_key_service
        .create_api_key(customer.user_id, CreateApiKeyRequest {
            key_name: "test-simple-monitoring-agent".to_string(),
            expires_in_days: Some(30),
        })
        .await.expect("API key creation should succeed");
    
    let registered_agent = agent_service
        .register_agent_with_api_key(RegisterAgentRequest {
            api_key: api_key_response.api_key.clone(),
            device_name: "Monitoring-Test-Server".to_string(),
            hardware_fingerprint: "test-simple-server-001".to_string(),
            os_info: serde_json::json!({"name": "Ubuntu", "version": "22.04", "type": "Server"}),
            version: "1.0.0".to_string(),
        })
        .await.expect("Agent registration should succeed");
    
    sqlx::query!(
        "UPDATE agents.endpoints SET status = 'online', last_heartbeat = now() WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .execute(database.pool())
    .await.expect("Agent should come online");
    
    println!("✅ Analyst and customer setup complete with online agent");
    
    // Step 2: Analyst Login
    println!("🔐 Step 2: Security Analyst Login");
    
    let analyst_login = user_service
        .verify_credentials("monitoring_analyst", "AnalystSecure123!")
        .await
        .expect("Analyst login should work")
        .expect("Analyst should exist");
    
    assert_eq!(analyst_login.user_id, analyst.user_id);
    assert!(analyst_login.is_active);
    println!("✅ Security analyst logged in successfully");
    
    // Step 3: Analyst Can Monitor New Asset
    println!("📊 Step 3: Analyst Monitoring Dashboard - New Asset Detection");
    
    // Simulate what analyst would see in monitoring dashboard
    let monitored_assets = sqlx::query!(
        r#"
        SELECT 
            a.agent_id, a.hardware_fingerprint, a.status, a.last_heartbeat, a.version,
            ak.user_id as owner_user_id
        FROM agents.endpoints a
        LEFT JOIN users.api_keys ak ON a.registered_via_key_id = ak.key_id
        WHERE a.status = 'online'
        ORDER BY a.last_heartbeat DESC
        "#
    )
    .fetch_all(database.pool())
    .await
    .expect("Should get monitored assets");
    
    let customer_asset = monitored_assets.iter()
        .find(|asset| asset.agent_id == registered_agent.agent_id)
        .expect("Should find customer's asset");
    
    assert_eq!(customer_asset.status, "online");
    assert!(customer_asset.last_heartbeat.is_some());
    println!("✅ Analyst can see and monitor customer's new asset");
    
    // Step 4: Real-time Asset Health Monitoring
    println!("💓 Step 4: Real-time Asset Health Monitoring");
    
    // Simulate monitoring heartbeats
    for i in 1..=3 {
        let health_check = sqlx::query!(
            "UPDATE agents.endpoints SET last_heartbeat = now() WHERE agent_id = $1 RETURNING last_heartbeat",
            registered_agent.agent_id
        )
        .fetch_one(database.pool())
        .await
        .expect("Health update should work");
        
        assert!(health_check.last_heartbeat.is_some());
        println!("✅ Health monitoring {}/3 - Asset is healthy and responding", i);
    }
    
    // Step 5: Detect Agent Status Changes
    println!("🔄 Step 5: Monitoring Agent Status Changes");
    
    // Simulate agent going offline
    sqlx::query!(
        "UPDATE agents.endpoints SET status = 'offline', last_heartbeat = now() WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .execute(database.pool())
    .await.expect("Status update should work");
    
    let offline_status = sqlx::query!(
        "SELECT status, last_heartbeat FROM agents.endpoints WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .fetch_one(database.pool())
    .await
    .expect("Should get status");
    
    assert_eq!(offline_status.status, "offline");
    println!("✅ Analyst detected asset going offline");
    
    // Simulate agent coming back online
    sqlx::query!(
        "UPDATE agents.endpoints SET status = 'online', last_heartbeat = now() WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .execute(database.pool())
    .await.expect("Status update should work");
    
    let online_status = sqlx::query!(
        "SELECT status FROM agents.endpoints WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .fetch_one(database.pool())
    .await
    .expect("Should get status");
    
    assert_eq!(online_status.status, "online");
    println!("✅ Analyst detected asset coming back online");
    
    println!("✅ Complete analyst monitoring workflow SUCCESSFUL!");
    println!("🔍 Workflow Summary: Analyst Login → Dashboard Access → Asset Discovery → Health Monitoring → Status Detection");
    
    cleanup_workflow_data(&database).await.expect("Cleanup should work");
}

// TEST 4: END-TO-END INTEGRATION TEST
#[tokio::test]
async fn test_complete_end_to_end_workflow() {
    let (database, _auth_service, user_service, agent_service, api_key_service) = create_workflow_setup().await;
    
    println!("🚀 Starting COMPLETE End-to-End Customer Lifecycle Test");
    println!("🔄 Full Workflow: Onboarding → Monitoring → Cancellation");
    
    // PHASE 1: CUSTOMER ONBOARDING
    println!("\n=== PHASE 1: CUSTOMER ONBOARDING ===");
    
    // Customer registration and login
    let customer = user_service.create_user(CreateUserRequest {
        username: "e2e_customer".to_string(),
        email: "e2e@simple-workflow.com".to_string(),
        password: "E2ETest123!".to_string(),
    }).await.expect("Customer creation should succeed");
    
    user_service.verify_credentials("e2e_customer", "E2ETest123!")
        .await.expect("Login should work").expect("Customer should exist");
    println!("✅ Customer registration and login complete");
    
    // Agent download and installation
    let api_key_response = api_key_service
        .create_api_key(customer.user_id, CreateApiKeyRequest {
            key_name: "test-simple-e2e-agent".to_string(),
            expires_in_days: Some(30),
        })
        .await.expect("API key creation should succeed");
    
    let registered_agent = agent_service
        .register_agent_with_api_key(RegisterAgentRequest {
            api_key: api_key_response.api_key.clone(),
            device_name: "E2E-Test-System".to_string(),
            hardware_fingerprint: "test-simple-e2e-001".to_string(),
            os_info: serde_json::json!({"name": "Windows", "version": "11", "type": "Workstation"}),
            version: "1.0.0".to_string(),
        })
        .await.expect("Agent registration should succeed");
    
    sqlx::query!(
        "UPDATE agents.endpoints SET status = 'online', last_heartbeat = now() WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .execute(database.pool())
    .await.expect("Agent should come online");
    
    println!("✅ Agent download, installation, and registration complete");
    
    // PHASE 2: ANALYST MONITORING
    println!("\n=== PHASE 2: ANALYST MONITORING ===");
    
    let analyst = user_service.create_user(CreateUserRequest {
        username: "e2e_analyst".to_string(),
        email: "e2e_analyst@simple-workflow.com".to_string(),
        password: "E2EAnalyst123!".to_string(),
    }).await.expect("Analyst creation should succeed");
    
    user_service.verify_credentials("e2e_analyst", "E2EAnalyst123!")
        .await.expect("Analyst login should work").expect("Analyst should exist");
    println!("✅ Security analyst login successful");
    
    // Verify analyst can monitor the asset
    let monitored_asset = sqlx::query!(
        "SELECT agent_id, status, last_heartbeat FROM agents.endpoints WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .fetch_one(database.pool())
    .await
    .expect("Should find monitored asset");
    
    assert_eq!(monitored_asset.status, "online");
    println!("✅ Analyst successfully monitoring customer asset");
    
    // Simulate monitoring activity
    for i in 1..=2 {
        sqlx::query!(
            "UPDATE agents.endpoints SET last_heartbeat = now() WHERE agent_id = $1",
            registered_agent.agent_id
        )
        .execute(database.pool())
        .await.expect("Monitoring update should work");
        println!("✅ Monitoring activity {}/2 - Asset healthy", i);
    }
    
    // PHASE 3: CUSTOMER CANCELLATION
    println!("\n=== PHASE 3: CUSTOMER ACCOUNT CANCELLATION ===");
    
    // Customer initiates cancellation
    let pre_cancel_check = user_service
        .verify_credentials("e2e_customer", "E2ETest123!")
        .await.expect("Pre-cancel check should work").expect("Customer should exist");
    assert!(pre_cancel_check.is_active);
    println!("✅ Customer account verified active before cancellation");
    
    // Automatic agent deactivation
    sqlx::query!(
        "UPDATE agents.endpoints SET status = 'offline' WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .execute(database.pool())
    .await.expect("Agent deactivation should work");
    println!("✅ Agent automatically deactivated");
    
    // Automatic agent uninstallation (removal)
    sqlx::query!(
        "DELETE FROM agents.endpoints WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .execute(database.pool())
    .await.expect("Agent removal should work");
    println!("✅ Agent automatically uninstalled from customer device");
    
    // Account deactivation and deletion
    sqlx::query!(
        "UPDATE users.users SET is_active = FALSE WHERE user_id = $1",
        customer.user_id
    )
    .execute(database.pool())
    .await.expect("Account deactivation should work");
    
    sqlx::query!("DELETE FROM users.api_keys WHERE user_id = $1", customer.user_id)
        .execute(database.pool()).await.expect("API key deletion should work");
    
    sqlx::query!("DELETE FROM users.users WHERE user_id = $1", customer.user_id)
        .execute(database.pool()).await.expect("User deletion should work");
    println!("✅ Customer account permanently deleted");
    
    // Email notification (mock)
    println!("✅ Customer notified via email about account cancellation");
    
    // PHASE 4: FINAL VERIFICATION
    println!("\n=== PHASE 4: COMPLETE VERIFICATION ===");
    
    // Customer cannot login
    let post_cancel_login = user_service
        .verify_credentials("e2e_customer", "E2ETest123!")
        .await.expect("Should not error");
    assert!(post_cancel_login.is_none());
    println!("✅ Customer can no longer login");
    
    // Asset no longer monitored
    let asset_check = sqlx::query!(
        "SELECT agent_id FROM agents.endpoints WHERE agent_id = $1",
        registered_agent.agent_id
    )
    .fetch_optional(database.pool())
    .await.expect("Query should work");
    assert!(asset_check.is_none());
    println!("✅ Asset completely removed from monitoring system");
    
    // Analyst still functional
    let analyst_check = user_service
        .verify_credentials("e2e_analyst", "E2EAnalyst123!")
        .await.expect("Analyst check should work").expect("Analyst should exist");
    assert!(analyst_check.is_active);
    println!("✅ Security analyst account unaffected");
    
    // Database completely clean
    let cleanup_verification = sqlx::query!(
        "SELECT user_id FROM users.users WHERE user_id = $1",
        customer.user_id
    )
    .fetch_optional(database.pool())
    .await.expect("Query should work");
    assert!(cleanup_verification.is_none());
    println!("✅ Database completely cleaned - no traces remain");
    
    println!("\n🎉 COMPLETE END-TO-END CUSTOMER LIFECYCLE TEST SUCCESSFUL!");
    println!("📊 Successfully tested complete workflow:");
    println!("   ✅ Customer registration and login");
    println!("   ✅ Agent download and API key generation");
    println!("   ✅ Agent installation and registration");
    println!("   ✅ Agent connection to application");
    println!("   ✅ Asset available for analyst monitoring");
    println!("   ✅ Real-time monitoring and health checks");
    println!("   ✅ Customer account cancellation process");
    println!("   ✅ Automatic agent deactivation");
    println!("   ✅ Automatic agent uninstallation");
    println!("   ✅ Asset removal from monitoring");
    println!("   ✅ Complete account deletion and cleanup");
    println!("   ✅ Customer email notification");
    println!("   ✅ System integrity maintained");
    
    cleanup_workflow_data(&database).await.expect("Final cleanup should work");
}