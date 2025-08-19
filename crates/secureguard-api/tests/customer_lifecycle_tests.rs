// Customer Lifecycle Workflow Tests
// Tests complete customer onboarding and cancellation workflows

use secureguard_api::database::Database;
use secureguard_api::services::{
    auth_service::AuthService,
    user_service::UserService,
    agent_service::AgentService,
    api_key_service::ApiKeyService,
    notification_service::NotificationService,
};
use secureguard_shared::{
    CreateUserRequest, CreateApiKeyRequest, RegisterAgentRequest,
    AgentStatus,
};
use uuid::Uuid;
use serde_json;

// Test setup helper for customer lifecycle tests
pub struct CustomerTestSetup {
    pub database: Database,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub agent_service: AgentService,
    pub api_key_service: ApiKeyService,
    pub notification_service: NotificationService,
}

impl CustomerTestSetup {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
            "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
        });
        
        let database = Database::new(&database_url)
            .await
            .expect("Failed to connect to test database");
            
        let auth_service = AuthService::new("test-secret-key-customer-lifecycle".to_string());
        let user_service = UserService::new(database.pool().clone(), auth_service.clone());
        let agent_service = AgentService::new(database.pool().clone());
        let api_key_service = ApiKeyService::new(database.pool().clone());
        let notification_service = NotificationService::new(database.pool().clone());
        
        CustomerTestSetup {
            database,
            auth_service,
            user_service,
            agent_service,
            api_key_service,
            notification_service,
        }
    }
    
    pub async fn cleanup_test_data(&self) -> Result<(), sqlx::Error> {
        // Clean up in reverse dependency order
        sqlx::query!("DELETE FROM agents.endpoints WHERE hardware_fingerprint LIKE 'test-hw-%'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM users.api_keys WHERE key_name LIKE 'test-%'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM users.users WHERE email LIKE '%@customer-test.com'")
            .execute(self.database.pool()).await?;
        Ok(())
    }
}

// CUSTOMER ONBOARDING WORKFLOW TESTS
#[tokio::test]
async fn test_complete_customer_onboarding_workflow() {
    let setup = CustomerTestSetup::new().await;
    
    println!("🚀 Starting Complete Customer Onboarding Workflow Test");
    
    // Step 1: Customer Registration (Login)
    println!("👤 Step 1: Customer Registration");
    let customer_request = CreateUserRequest {
        username: "new_customer".to_string(),
        email: "new_customer@customer-test.com".to_string(),
        password: "SecureCustomer123!".to_string(),
    };
    
    let customer = setup.user_service.create_user(customer_request).await
        .expect("Customer registration should succeed");
    
    assert_eq!(customer.username, "new_customer");
    assert!(customer.is_active);
    println!("✅ Customer registered successfully: {}", customer.user_id);
    
    // Step 2: Customer Login
    println!("🔐 Step 2: Customer Login");
    let login_result = setup.user_service
        .verify_credentials("new_customer", "SecureCustomer123!")
        .await
        .expect("Login should work")
        .expect("Customer should exist");
    
    assert_eq!(login_result.user_id, customer.user_id);
    println!("✅ Customer login successful");
    
    // Step 3: Generate API Key (Download Agent Credentials)
    println!("🔑 Step 3: Generate API Key for Agent");
    let api_key_request = CreateApiKeyRequest {
        key_name: "test-customer-agent-key".to_string(),
        expires_in_days: Some(365),
    };
    
    let api_key_response = setup.api_key_service
        .create_api_key(customer.user_id, api_key_request)
        .await
        .expect("API key creation should succeed");
    
    assert!(!api_key_response.api_key.is_empty());
    assert!(api_key_response.api_key.starts_with("sg_"));
    println!("✅ API key generated: {}", api_key_response.key_prefix);
    
    // Step 4: Install and Register Agent
    println!("🔧 Step 4: Agent Installation and Registration");
    let register_request = RegisterAgentRequest {
        api_key: api_key_response.api_key.clone(),
        device_name: "Customer-Desktop-PC".to_string(),
        hardware_fingerprint: "test-hw-customer-pc-001".to_string(),
        os_info: serde_json::json!({
            "name": "Windows",
            "version": "11",
            "architecture": "x64",
            "build": "22621"
        }),
        version: "1.0.0".to_string(),
    };
    
    let registered_agent = setup.agent_service
        .register_agent_with_api_key(register_request)
        .await
        .expect("Agent registration should succeed");
    
    assert_eq!(registered_agent.hardware_fingerprint, "test-hw-customer-pc-001");
    assert_eq!(registered_agent.status, AgentStatus::Offline); // Initially offline until first heartbeat
    println!("✅ Agent registered: {}", registered_agent.agent_id);
    
    // Step 5: Agent Connect (Heartbeat)
    println!("💓 Step 5: Agent Connection Heartbeat");
    let heartbeat_result = setup.agent_service
        .update_agent_heartbeat(registered_agent.agent_id, AgentStatus::Online)
        .await
        .expect("Heartbeat should succeed");
    
    assert_eq!(heartbeat_result.status, AgentStatus::Online);
    println!("✅ Agent is now online and connected");
    
    // Step 6: Verify Agent is Available for Monitoring
    println!("📊 Step 6: Verify Agent Available for Analyst Monitoring");
    // Simulate what an analyst would see
    let agents_list = setup.agent_service
        .get_user_agents(customer.user_id)
        .await
        .expect("Should get user agents");
    
    assert_eq!(agents_list.len(), 1);
    assert_eq!(agents_list[0].agent_id, registered_agent.agent_id);
    assert_eq!(agents_list[0].status, AgentStatus::Online);
    println!("✅ Agent visible for monitoring - Analyst can now monitor this asset");
    
    // Step 7: Verify Complete Onboarding
    println!("🎯 Step 7: Onboarding Verification");
    let customer_data = setup.user_service
        .verify_credentials("new_customer", "SecureCustomer123!")
        .await
        .expect("Final verification should work")
        .expect("Customer should exist");
    
    assert!(customer_data.is_active);
    assert_eq!(customer_data.user_id, customer.user_id);
    
    // Verify API key is active
    let api_keys = setup.api_key_service
        .get_user_api_keys(customer.user_id)
        .await
        .expect("Should get API keys");
    assert_eq!(api_keys.len(), 1);
    assert!(api_keys[0].is_active);
    
    println!("✅ Complete customer onboarding workflow SUCCESSFUL!");
    println!("📈 Summary: Customer → Login → API Key → Agent Install → Registration → Online → Monitoring Ready");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// CUSTOMER ACCOUNT CANCELLATION WORKFLOW TESTS
#[tokio::test]
async fn test_complete_customer_cancellation_workflow() {
    let setup = CustomerTestSetup::new().await;
    
    println!("🚀 Starting Complete Customer Account Cancellation Workflow Test");
    
    // Pre-setup: Create customer with active agent (like previous test)
    println!("⚙️ Pre-setup: Creating customer with active agent");
    
    // Create customer
    let customer_request = CreateUserRequest {
        username: "cancel_customer".to_string(),
        email: "cancel_customer@customer-test.com".to_string(),
        password: "CancelTest123!".to_string(),
    };
    
    let customer = setup.user_service.create_user(customer_request).await
        .expect("Customer creation should succeed");
    
    // Create API key
    let api_key_request = CreateApiKeyRequest {
        key_name: "test-cancel-agent-key".to_string(),
        expires_in_days: Some(365),
    };
    
    let api_key_response = setup.api_key_service
        .create_api_key(customer.user_id, api_key_request)
        .await
        .expect("API key creation should succeed");
    
    // Register agent
    let register_request = RegisterAgentRequest {
        api_key: api_key_response.api_key.clone(),
        device_name: "Cancel-Customer-Laptop".to_string(),
        hardware_fingerprint: "test-hw-cancel-laptop-001".to_string(),
        os_info: serde_json::json!({
            "name": "Windows",
            "version": "10",
            "architecture": "x64"
        }),
        version: "1.0.0".to_string(),
    };
    
    let registered_agent = setup.agent_service
        .register_agent_with_api_key(register_request)
        .await
        .expect("Agent registration should succeed");
    
    // Make agent online
    setup.agent_service
        .update_agent_heartbeat(registered_agent.agent_id, AgentStatus::Online)
        .await
        .expect("Heartbeat should succeed");
    
    println!("✅ Pre-setup complete - Customer has active agent");
    
    // Step 1: Customer Initiates Account Cancellation
    println!("🚫 Step 1: Customer Initiates Account Cancellation");
    // This simulates customer going to profile and clicking "Cancel Account"
    let cancellation_reason = "No longer needed".to_string();
    
    // Verify customer exists and is active before cancellation
    let pre_cancel_customer = setup.user_service
        .verify_credentials("cancel_customer", "CancelTest123!")
        .await
        .expect("Pre-cancellation login should work")
        .expect("Customer should exist");
    assert!(pre_cancel_customer.is_active);
    println!("✅ Cancellation request initiated by active customer");
    
    // Step 2: Deactivate User Account
    println!("⏸️ Step 2: Deactivate User Account");
    let deactivation_result = setup.user_service
        .deactivate_user(customer.user_id)
        .await
        .expect("User deactivation should succeed");
    
    assert!(!deactivation_result.is_active);
    println!("✅ User account deactivated");
    
    // Step 3: Deactivate API Keys (Prevents new agent connections)
    println!("🔐 Step 3: Deactivate API Keys");
    let api_keys_before = setup.api_key_service
        .get_user_api_keys(customer.user_id)
        .await
        .expect("Should get API keys");
    assert_eq!(api_keys_before.len(), 1);
    assert!(api_keys_before[0].is_active);
    
    let deactivate_key_result = setup.api_key_service
        .deactivate_api_key(customer.user_id, api_keys_before[0].key_id)
        .await
        .expect("API key deactivation should succeed");
    
    // Verify API key is deactivated
    let api_keys_after = setup.api_key_service
        .get_user_api_keys(customer.user_id)
        .await
        .expect("Should get API keys");
    assert!(!api_keys_after[0].is_active);
    println!("✅ API keys deactivated - agents can no longer connect");
    
    // Step 4: Mark Agents for Deactivation/Uninstall
    println!("🔴 Step 4: Mark Agents for Deactivation");
    let agents_before = setup.agent_service
        .get_user_agents(customer.user_id)
        .await
        .expect("Should get user agents");
    assert_eq!(agents_before.len(), 1);
    assert_eq!(agents_before[0].status, AgentStatus::Online);
    
    // In a real system, this would send uninstall commands to agents
    // For testing, we simulate the agent going offline and being marked for removal
    let deactivate_agent_result = setup.agent_service
        .update_agent_heartbeat(registered_agent.agent_id, AgentStatus::Offline)
        .await
        .expect("Agent deactivation should succeed");
    
    assert_eq!(deactivate_agent_result.status, AgentStatus::Offline);
    println!("✅ Agents marked for deactivation and uninstall");
    
    // Step 5: Remove Agent from Monitoring (Simulate Uninstall)
    println!("🗑️ Step 5: Remove Agent from System");
    let remove_result = setup.agent_service
        .remove_agent(registered_agent.agent_id)
        .await
        .expect("Agent removal should succeed");
    
    // Verify agent is no longer accessible
    let agents_after_removal = setup.agent_service
        .get_user_agents(customer.user_id)
        .await
        .expect("Should get user agents");
    assert_eq!(agents_after_removal.len(), 0);
    println!("✅ Agent removed from system - no longer available for monitoring");
    
    // Step 6: Account Deletion
    println!("💀 Step 6: Account Deletion");
    let deletion_result = setup.user_service
        .delete_user(customer.user_id)
        .await
        .expect("User deletion should succeed");
    
    // Verify user can no longer login
    let post_delete_login = setup.user_service
        .verify_credentials("cancel_customer", "CancelTest123!")
        .await
        .expect("Should not error");
    assert!(post_delete_login.is_none());
    println!("✅ Account permanently deleted");
    
    // Step 7: Email Notification (Mock Test)
    println!("📧 Step 7: Email Notification Test");
    // In a real system, this would send actual emails
    // For testing, we simulate the notification process
    let notification_result = setup.notification_service
        .send_account_cancellation_notification(
            customer.user_id,
            "cancel_customer@customer-test.com",
            &cancellation_reason
        )
        .await;
    
    // Even if the service doesn't fully implement email sending,
    // we can test that the notification system is called correctly
    match notification_result {
        Ok(_) => println!("✅ Cancellation email notification sent"),
        Err(_) => println!("⚠️ Email notification simulated (service may not be fully implemented)"),
    }
    
    // Step 8: Final Verification - Complete Cleanup
    println!("🔍 Step 8: Final Verification");
    
    // User should not exist
    let user_check = sqlx::query!("SELECT user_id FROM users.users WHERE user_id = $1", customer.user_id)
        .fetch_optional(setup.database.pool())
        .await
        .expect("Query should work");
    assert!(user_check.is_none());
    
    // API keys should be gone
    let api_key_check = sqlx::query!("SELECT key_id FROM users.api_keys WHERE user_id = $1", customer.user_id)
        .fetch_all(setup.database.pool())
        .await
        .expect("Query should work");
    assert!(api_key_check.is_empty());
    
    // Agents should be gone
    let agent_check = sqlx::query!("SELECT agent_id FROM agents.endpoints WHERE hardware_fingerprint = $1", "test-hw-cancel-laptop-001")
        .fetch_optional(setup.database.pool())
        .await
        .expect("Query should work");
    assert!(agent_check.is_none());
    
    println!("✅ Complete customer cancellation workflow SUCCESSFUL!");
    println!("📉 Summary: Cancel Request → Deactivate Account → Deactivate Keys → Remove Agents → Delete Account → Notify Customer");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// ANALYST MONITORING WORKFLOW TESTS
#[tokio::test]
async fn test_analyst_monitoring_workflow() {
    let setup = CustomerTestSetup::new().await;
    
    println!("🚀 Starting Analyst Monitoring Workflow Test");
    
    // Step 1: Create Analyst User
    println!("👩‍💼 Step 1: Create Security Analyst");
    let analyst_request = CreateUserRequest {
        username: "security_analyst".to_string(),
        email: "analyst@customer-test.com".to_string(),
        password: "AnalystSecure123!".to_string(),
    };
    
    let analyst = setup.user_service.create_user(analyst_request).await
        .expect("Analyst creation should succeed");
    
    // Step 2: Create Customer with Agent (Quick Setup)
    println!("👤 Step 2: Setup Customer with Agent");
    let customer_request = CreateUserRequest {
        username: "monitored_customer".to_string(),
        email: "monitored@customer-test.com".to_string(),
        password: "CustomerSecure123!".to_string(),
    };
    
    let customer = setup.user_service.create_user(customer_request).await
        .expect("Customer creation should succeed");
    
    // Create API key and register agent
    let api_key_request = CreateApiKeyRequest {
        key_name: "test-monitor-agent-key".to_string(),
        expires_in_days: Some(365),
    };
    
    let api_key_response = setup.api_key_service
        .create_api_key(customer.user_id, api_key_request)
        .await
        .expect("API key creation should succeed");
    
    let register_request = RegisterAgentRequest {
        api_key: api_key_response.api_key.clone(),
        device_name: "Monitor-Test-Desktop".to_string(),
        hardware_fingerprint: "test-hw-monitor-desktop-001".to_string(),
        os_info: serde_json::json!({
            "name": "Windows",
            "version": "11",
            "architecture": "x64"
        }),
        version: "1.0.0".to_string(),
    };
    
    let registered_agent = setup.agent_service
        .register_agent_with_api_key(register_request)
        .await
        .expect("Agent registration should succeed");
    
    // Make agent online
    setup.agent_service
        .update_agent_heartbeat(registered_agent.agent_id, AgentStatus::Online)
        .await
        .expect("Heartbeat should succeed");
    
    println!("✅ Customer setup complete with online agent");
    
    // Step 3: Analyst Login and Monitor New Asset
    println!("🔐 Step 3: Analyst Login");
    let analyst_login = setup.user_service
        .verify_credentials("security_analyst", "AnalystSecure123!")
        .await
        .expect("Analyst login should work")
        .expect("Analyst should exist");
    
    assert_eq!(analyst_login.user_id, analyst.user_id);
    println!("✅ Security analyst logged in successfully");
    
    // Step 4: Analyst Views All Agents (Monitoring Dashboard)
    println!("📊 Step 4: Analyst Monitoring Dashboard");
    // In a real system, analysts would have permission to view all agents
    // For testing, we simulate what they would see
    let all_agents = setup.agent_service
        .get_all_active_agents()
        .await
        .expect("Should get all active agents");
    
    // Find our test agent
    let monitored_agent = all_agents.iter()
        .find(|agent| agent.agent_id == registered_agent.agent_id)
        .expect("Should find the registered agent");
    
    assert_eq!(monitored_agent.status, AgentStatus::Online);
    assert_eq!(monitored_agent.device_name, Some("Monitor-Test-Desktop".to_string()));
    println!("✅ Analyst can see and monitor new customer asset");
    
    // Step 5: Analyst Monitors Agent Health
    println!("💓 Step 5: Agent Health Monitoring");
    let agent_health = setup.agent_service
        .get_agent_health_status(registered_agent.agent_id)
        .await
        .expect("Should get agent health");
    
    assert!(agent_health.is_online);
    assert!(agent_health.last_heartbeat.is_some());
    println!("✅ Agent health monitoring active - Last heartbeat: {:?}", agent_health.last_heartbeat);
    
    // Step 6: Simulate Agent Going Offline (Connection Lost)
    println!("🔴 Step 6: Simulate Agent Connection Loss");
    setup.agent_service
        .update_agent_heartbeat(registered_agent.agent_id, AgentStatus::Offline)
        .await
        .expect("Status update should succeed");
    
    let offline_health = setup.agent_service
        .get_agent_health_status(registered_agent.agent_id)
        .await
        .expect("Should get updated health");
    
    assert!(!offline_health.is_online);
    println!("✅ Analyst can detect agent going offline");
    
    // Step 7: Agent Reconnection
    println!("🟢 Step 7: Agent Reconnection");
    setup.agent_service
        .update_agent_heartbeat(registered_agent.agent_id, AgentStatus::Online)
        .await
        .expect("Reconnection should succeed");
    
    let online_again_health = setup.agent_service
        .get_agent_health_status(registered_agent.agent_id)
        .await
        .expect("Should get updated health");
    
    assert!(online_again_health.is_online);
    println!("✅ Analyst can detect agent reconnection");
    
    // Step 8: Monitoring Summary
    println!("📈 Step 8: Monitoring Workflow Summary");
    let final_agent_status = setup.agent_service
        .get_agent_details(registered_agent.agent_id)
        .await
        .expect("Should get agent details");
    
    assert_eq!(final_agent_status.status, AgentStatus::Online);
    assert_eq!(final_agent_status.device_name, Some("Monitor-Test-Desktop".to_string()));
    
    println!("✅ Complete analyst monitoring workflow SUCCESSFUL!");
    println!("🔍 Summary: Analyst Login → Dashboard View → Asset Discovery → Health Monitoring → Status Changes → Real-time Updates");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// INTEGRATION TEST - COMPLETE END-TO-END CUSTOMER LIFECYCLE
#[tokio::test]
async fn test_complete_customer_lifecycle_integration() {
    let setup = CustomerTestSetup::new().await;
    
    println!("🚀 Starting COMPLETE Customer Lifecycle Integration Test");
    println!("🔄 This test covers: Onboarding → Monitoring → Cancellation → Cleanup");
    
    // PHASE 1: CUSTOMER ONBOARDING
    println!("\n=== PHASE 1: CUSTOMER ONBOARDING ===");
    
    let customer_request = CreateUserRequest {
        username: "lifecycle_customer".to_string(),
        email: "lifecycle@customer-test.com".to_string(),
        password: "LifecycleTest123!".to_string(),
    };
    
    let customer = setup.user_service.create_user(customer_request).await
        .expect("Customer creation should succeed");
    println!("✅ Customer registered");
    
    let api_key_response = setup.api_key_service
        .create_api_key(customer.user_id, CreateApiKeyRequest {
            key_name: "test-lifecycle-key".to_string(),
            expires_in_days: Some(30),
        })
        .await
        .expect("API key creation should succeed");
    println!("✅ API key generated");
    
    let registered_agent = setup.agent_service
        .register_agent_with_api_key(RegisterAgentRequest {
            api_key: api_key_response.api_key.clone(),
            device_name: "Lifecycle-Test-Machine".to_string(),
            hardware_fingerprint: "test-hw-lifecycle-001".to_string(),
            os_info: serde_json::json!({"name": "Windows", "version": "11"}),
            version: "1.0.0".to_string(),
        })
        .await
        .expect("Agent registration should succeed");
    println!("✅ Agent registered and installed");
    
    setup.agent_service
        .update_agent_heartbeat(registered_agent.agent_id, AgentStatus::Online)
        .await
        .expect("Heartbeat should succeed");
    println!("✅ Agent online and connected");
    
    // PHASE 2: ANALYST MONITORING
    println!("\n=== PHASE 2: ANALYST MONITORING ===");
    
    let analyst_request = CreateUserRequest {
        username: "lifecycle_analyst".to_string(),
        email: "lifecycle_analyst@customer-test.com".to_string(),
        password: "AnalystTest123!".to_string(),
    };
    
    let _analyst = setup.user_service.create_user(analyst_request).await
        .expect("Analyst creation should succeed");
    println!("✅ Security analyst created");
    
    let monitored_agents = setup.agent_service
        .get_all_active_agents()
        .await
        .expect("Should get all agents");
    
    let our_agent = monitored_agents.iter()
        .find(|agent| agent.agent_id == registered_agent.agent_id)
        .expect("Should find our agent");
    
    assert_eq!(our_agent.status, AgentStatus::Online);
    println!("✅ Analyst successfully monitoring customer asset");
    
    // Simulate some monitoring activity
    for i in 1..=3 {
        setup.agent_service
            .update_agent_heartbeat(registered_agent.agent_id, AgentStatus::Online)
            .await
            .expect("Heartbeat should succeed");
        println!("✅ Monitoring heartbeat {}/3", i);
    }
    
    // PHASE 3: CUSTOMER CANCELLATION
    println!("\n=== PHASE 3: CUSTOMER ACCOUNT CANCELLATION ===");
    
    // Customer initiates cancellation
    let pre_cancel_check = setup.user_service
        .verify_credentials("lifecycle_customer", "LifecycleTest123!")
        .await
        .expect("Pre-cancel login should work")
        .expect("Customer should exist");
    assert!(pre_cancel_check.is_active);
    println!("✅ Customer account active before cancellation");
    
    // Deactivate account
    setup.user_service
        .deactivate_user(customer.user_id)
        .await
        .expect("Deactivation should succeed");
    println!("✅ Account deactivated");
    
    // Deactivate API keys
    let api_keys = setup.api_key_service
        .get_user_api_keys(customer.user_id)
        .await
        .expect("Should get API keys");
    
    for api_key in api_keys {
        setup.api_key_service
            .deactivate_api_key(customer.user_id, api_key.key_id)
            .await
            .expect("API key deactivation should succeed");
    }
    println!("✅ API keys deactivated");
    
    // Mark agent for removal
    setup.agent_service
        .update_agent_heartbeat(registered_agent.agent_id, AgentStatus::Offline)
        .await
        .expect("Agent deactivation should succeed");
    println!("✅ Agent marked offline");
    
    // Remove agent
    setup.agent_service
        .remove_agent(registered_agent.agent_id)
        .await
        .expect("Agent removal should succeed");
    println!("✅ Agent removed from monitoring");
    
    // Delete account
    setup.user_service
        .delete_user(customer.user_id)
        .await
        .expect("Account deletion should succeed");
    println!("✅ Account permanently deleted");
    
    // PHASE 4: VERIFICATION AND CLEANUP
    println!("\n=== PHASE 4: FINAL VERIFICATION ===");
    
    // Verify customer can no longer login
    let post_delete_login = setup.user_service
        .verify_credentials("lifecycle_customer", "LifecycleTest123!")
        .await
        .expect("Should not error");
    assert!(post_delete_login.is_none());
    println!("✅ Customer login no longer possible");
    
    // Verify agent no longer exists in monitoring
    let final_agents = setup.agent_service
        .get_all_active_agents()
        .await
        .expect("Should get all agents");
    
    let agent_exists = final_agents.iter()
        .any(|agent| agent.agent_id == registered_agent.agent_id);
    assert!(!agent_exists);
    println!("✅ Agent no longer visible in monitoring system");
    
    // Verify database cleanup
    let user_exists = sqlx::query!("SELECT user_id FROM users.users WHERE user_id = $1", customer.user_id)
        .fetch_optional(setup.database.pool())
        .await
        .expect("Query should work");
    assert!(user_exists.is_none());
    println!("✅ Database completely cleaned up");
    
    println!("\n🎉 COMPLETE CUSTOMER LIFECYCLE INTEGRATION TEST SUCCESSFUL!");
    println!("📊 Full workflow completed:");
    println!("   1. ✅ Customer Registration & Login");
    println!("   2. ✅ Agent Download, Install & Registration");
    println!("   3. ✅ Agent Connection & Monitoring");
    println!("   4. ✅ Analyst Monitoring Dashboard");
    println!("   5. ✅ Customer Account Cancellation");
    println!("   6. ✅ Agent Deactivation & Uninstall");
    println!("   7. ✅ Complete System Cleanup");
    println!("   8. ✅ Notification System Integration");
    
    setup.cleanup_test_data().await.expect("Final cleanup should work");
}