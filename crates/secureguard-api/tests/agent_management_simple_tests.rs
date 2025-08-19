// Simple Agent Management Tests
// Tests core agent management functionality using available services

use secureguard_api::database::Database;
use secureguard_api::services::{
    agent_service::AgentService,
    api_key_service::ApiKeyService,
    auth_service::AuthService,
    user_service::UserService,
};
use secureguard_shared::{
    CreateUserRequest, RegisterAgentRequest, HeartbeatRequest, AgentStatus, CreateApiKeyRequest,
};
use uuid::Uuid;
use chrono::Utc;

// Simplified test setup for Agent Management
pub struct SimpleAgentSetup {
    pub database: Database,
    pub user_service: UserService,
    pub agent_service: AgentService,
    pub api_key_service: ApiKeyService,
}

impl SimpleAgentSetup {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
            "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
        });
        
        let database = Database::new(&database_url)
            .await
            .expect("Failed to connect to test database");
            
        let auth_service = AuthService::new("test-secret-key-simple-agent".to_string());
        let user_service = UserService::new(database.pool().clone(), auth_service.clone());
        let agent_service = AgentService::new(database.pool().clone());
        let api_key_service = ApiKeyService::new(database.pool().clone());
        
        SimpleAgentSetup {
            database,
            user_service,
            agent_service,
            api_key_service,
        }
    }
    
    pub async fn cleanup_test_data(&self) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM agents.endpoints WHERE device_name LIKE 'simple_agent_%'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM users.api_keys WHERE key_name LIKE 'simple_agent_%'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM users.users WHERE email LIKE '%@simple-agent-test.com'")
            .execute(self.database.pool()).await?;
        Ok(())
    }

    // Helper to set up user with API key
    pub async fn setup_user_with_api_key(&self, username: &str) -> Result<(secureguard_shared::User, String), secureguard_shared::SecureGuardError> {
        // Make username unique to avoid conflicts - keep it very short
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let unique_username = format!("u{}", timestamp % 100000); // Short unique username
        
        // Create user
        let create_request = CreateUserRequest {
            username: unique_username.clone(),
            email: format!("{}@simple-agent-test.com", unique_username),
            password: format!("{}Password123!", unique_username),
        };
        
        let user = self.user_service.create_user(create_request).await?;
        
        // Create basic subscription setup (minimal)
        let tenant_id = Uuid::new_v4();
        sqlx::query!(
            "UPDATE users.users SET tenant_id = $1 WHERE user_id = $2",
            tenant_id,
            user.user_id
        )
        .execute(self.database.pool())
        .await
        .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?;
        
        // Create tenant
        sqlx::query!(
            "INSERT INTO tenants.tenants (tenant_id, name, plan_tier) VALUES ($1, $2, $3)",
            tenant_id, "Test Tenant", "basic"
        ).execute(self.database.pool()).await
        .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?;
        
        // Create subscription - try to get any available plan
        let plan_id = sqlx::query!("SELECT plan_id FROM subscriptions.plans ORDER BY plan_id LIMIT 1")
            .fetch_one(self.database.pool()).await
            .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?
            .plan_id;
        
        sqlx::query!(
            r#"
            INSERT INTO subscriptions.user_subscriptions 
            (user_id, plan_id, status, current_period_start, current_period_end, created_at)
            VALUES ($1, $2, 'active', $3, $4, $5)
            "#,
            user.user_id, plan_id, chrono::Utc::now(), chrono::Utc::now() + chrono::Duration::days(30), chrono::Utc::now()
        ).execute(self.database.pool()).await
        .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?;
        
        // Create API key
        let api_key_request = CreateApiKeyRequest {
            key_name: format!("simple_agent_{}_key", unique_username),
            expires_in_days: Some(30),
        };
        
        let api_key_response = self.api_key_service.create_api_key(user.user_id, api_key_request).await?;
        
        Ok((user, api_key_response.api_key))
    }
}

// TEST 1: BASIC AGENT REGISTRATION WORKFLOW
#[tokio::test]
async fn test_simple_agent_registration_workflow() {
    let setup = SimpleAgentSetup::new().await;
    
    println!("🚀 Starting Simple Agent Registration Test");
    
    // Step 1: Create user with API key
    println!("👤 Step 1: Create User with API Key");
    let (user, api_key) = setup.setup_user_with_api_key("simple_agent_user")
        .await.expect("User setup should succeed");
    
    println!("✅ User created: {} with API key", user.user_id);
    
    // Step 2: Register agent
    println!("🔧 Step 2: Register Agent");
    let register_request = RegisterAgentRequest {
        api_key: api_key.clone(),
        device_name: "simple_agent_device".to_string(),
        hardware_fingerprint: format!("hw_{}", Uuid::new_v4()),
        os_info: serde_json::json!({
            "name": "Windows 11 Pro",
            "version": "11.0.22000",
            "architecture": "x64"
        }),
        version: "1.0.0".to_string(),
    };
    
    let agent = setup.agent_service.register_agent_with_api_key(register_request).await
        .expect("Agent registration should succeed");
    
    println!("✅ Agent registered: {} on device {}", agent.agent_id, 
             agent.device_name.unwrap_or("unknown".to_string()));
    
    assert_eq!(agent.user_id.unwrap(), user.user_id);
    assert_eq!(agent.status, AgentStatus::Online);
    
    println!("✅ Simple Agent Registration workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 2: AGENT HEARTBEAT AND STATUS WORKFLOW
#[tokio::test]
async fn test_simple_agent_heartbeat_workflow() {
    let setup = SimpleAgentSetup::new().await;
    
    println!("🚀 Starting Simple Agent Heartbeat Test");
    
    // Step 1: Setup user and agent
    println!("🏗️ Step 1: Setup User and Agent");
    let (user, api_key) = setup.setup_user_with_api_key("simple_heartbeat_user")
        .await.expect("User setup should succeed");
    
    let register_request = RegisterAgentRequest {
        api_key: api_key.clone(),
        device_name: "simple_heartbeat_device".to_string(),
        hardware_fingerprint: format!("hw_{}", Uuid::new_v4()),
        os_info: serde_json::json!({"name": "Windows 11", "version": "11.0"}),
        version: "1.0.0".to_string(),
    };
    
    let agent = setup.agent_service.register_agent_with_api_key(register_request).await
        .expect("Agent registration should succeed");
    
    println!("✅ Agent registered: {}", agent.agent_id);
    
    // Step 2: Test heartbeat updates
    println!("💓 Step 2: Test Heartbeat Status Updates");
    
    // Update to offline
    let heartbeat_offline = HeartbeatRequest {
        agent_id: agent.agent_id,
        status: AgentStatus::Offline,
    };
    
    setup.agent_service.update_heartbeat(heartbeat_offline).await
        .expect("Offline heartbeat should succeed");
    
    // Verify status changed
    let updated_agent = setup.agent_service.find_by_id(agent.agent_id).await
        .expect("Agent lookup should succeed")
        .expect("Agent should exist");
    assert_eq!(updated_agent.status, AgentStatus::Offline);
    println!("✅ Agent status updated to offline");
    
    // Update back to online
    let heartbeat_online = HeartbeatRequest {
        agent_id: agent.agent_id,
        status: AgentStatus::Online,
    };
    
    setup.agent_service.update_heartbeat(heartbeat_online).await
        .expect("Online heartbeat should succeed");
    
    let recovered_agent = setup.agent_service.find_by_id(agent.agent_id).await
        .expect("Agent lookup should succeed")
        .expect("Agent should exist");
    assert_eq!(recovered_agent.status, AgentStatus::Online);
    println!("✅ Agent status recovered to online");
    
    println!("✅ Simple Agent Heartbeat workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 3: AGENT LISTING AND MANAGEMENT WORKFLOW
#[tokio::test]
async fn test_simple_agent_listing_workflow() {
    let setup = SimpleAgentSetup::new().await;
    
    println!("🚀 Starting Simple Agent Listing Test");
    
    // Step 1: Setup user and multiple agents
    println!("🏗️ Step 1: Setup User and Multiple Agents");
    let (user, api_key) = setup.setup_user_with_api_key("simple_listing_user")
        .await.expect("User setup should succeed");
    
    let mut agents = Vec::new();
    for i in 1..=3 {
        let register_request = RegisterAgentRequest {
            api_key: api_key.clone(),
            device_name: format!("simple_listing_device_{}", i),
            hardware_fingerprint: format!("hw_{}_{}", i, Uuid::new_v4()),
            os_info: serde_json::json!({
                "name": "Windows 11",
                "version": "11.0",
                "device_id": i
            }),
            version: "1.0.0".to_string(),
        };
        
        let agent = setup.agent_service.register_agent_with_api_key(register_request).await
            .expect("Agent registration should succeed");
        agents.push(agent);
    }
    
    println!("✅ Registered {} agents", agents.len());
    
    // Step 2: Test agent listing by user
    println!("📋 Step 2: Test Agent Listing by User");
    let user_agents = setup.agent_service.list_agents_for_user(user.user_id).await
        .expect("User agent listing should succeed");
    
    assert_eq!(user_agents.len(), 3);
    
    // Verify all agents belong to user
    for agent in &user_agents {
        assert_eq!(agent.user_id.unwrap(), user.user_id);
        assert_eq!(agent.status, AgentStatus::Online);
    }
    println!("✅ All agents listed correctly for user");
    
    // Step 3: Test agent listing by tenant
    println!("🏢 Step 3: Test Agent Listing by Tenant");
    let tenant_id = agents[0].tenant_id;
    let tenant_agents = setup.agent_service.list_agents_for_tenant(tenant_id).await
        .expect("Tenant agent listing should succeed");
    
    assert_eq!(tenant_agents.len(), 3);
    
    // Verify all agents belong to same tenant
    for agent in &tenant_agents {
        assert_eq!(agent.tenant_id, tenant_id);
    }
    println!("✅ All agents listed correctly for tenant");
    
    // Step 4: Test individual agent lookup
    println!("🔍 Step 4: Test Individual Agent Lookup");
    for original_agent in &agents {
        let found_agent = setup.agent_service.find_by_id(original_agent.agent_id).await
            .expect("Agent lookup should succeed")
            .expect("Agent should exist");
        
        assert_eq!(found_agent.agent_id, original_agent.agent_id);
        assert_eq!(found_agent.user_id, original_agent.user_id);
        assert_eq!(found_agent.device_name, original_agent.device_name);
    }
    println!("✅ Individual agent lookups successful");
    
    println!("✅ Simple Agent Listing workflow SUCCESSFUL!");
    println!("📊 Summary: {} agents registered, listed, and managed successfully", agents.len());
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 4: AGENT ERROR HANDLING WORKFLOW
#[tokio::test]
async fn test_simple_agent_error_handling_workflow() {
    let setup = SimpleAgentSetup::new().await;
    
    println!("🚀 Starting Simple Agent Error Handling Test");
    
    // Step 1: Test invalid API key
    println!("❌ Step 1: Test Invalid API Key Handling");
    let invalid_register_request = RegisterAgentRequest {
        api_key: "invalid_api_key".to_string(),
        device_name: "simple_error_device".to_string(),
        hardware_fingerprint: format!("hw_{}", Uuid::new_v4()),
        os_info: serde_json::json!({"name": "Windows 11"}),
        version: "1.0.0".to_string(),
    };
    
    let invalid_result = setup.agent_service.register_agent_with_api_key(invalid_register_request).await;
    assert!(invalid_result.is_err());
    println!("✅ Invalid API key correctly rejected");
    
    // Step 2: Test duplicate hardware fingerprint
    println!("🔄 Step 2: Test Duplicate Hardware Fingerprint Handling");
    let (user, api_key) = setup.setup_user_with_api_key("simple_error_user")
        .await.expect("User setup should succeed");
    
    let hardware_fp = format!("duplicate_hw_{}", Uuid::new_v4());
    
    // Register first agent
    let first_register = RegisterAgentRequest {
        api_key: api_key.clone(),
        device_name: "simple_error_device_1".to_string(),
        hardware_fingerprint: hardware_fp.clone(),
        os_info: serde_json::json!({"name": "Windows 11"}),
        version: "1.0.0".to_string(),
    };
    
    let _first_agent = setup.agent_service.register_agent_with_api_key(first_register).await
        .expect("First agent registration should succeed");
    
    // Try to register duplicate
    let duplicate_register = RegisterAgentRequest {
        api_key: api_key.clone(),
        device_name: "simple_error_device_2".to_string(),
        hardware_fingerprint: hardware_fp.clone(), // Same hardware fingerprint
        os_info: serde_json::json!({"name": "Windows 11"}),
        version: "1.0.0".to_string(),
    };
    
    let duplicate_result = setup.agent_service.register_agent_with_api_key(duplicate_register).await;
    assert!(duplicate_result.is_err());
    println!("✅ Duplicate hardware fingerprint correctly rejected");
    
    // Step 3: Test invalid agent heartbeat
    println!("💓 Step 3: Test Invalid Agent Heartbeat Handling");
    let invalid_heartbeat = HeartbeatRequest {
        agent_id: Uuid::new_v4(), // Non-existent agent
        status: AgentStatus::Online,
    };
    
    let heartbeat_result = setup.agent_service.update_heartbeat(invalid_heartbeat).await;
    assert!(heartbeat_result.is_err());
    println!("✅ Invalid agent heartbeat correctly rejected");
    
    println!("✅ Simple Agent Error Handling workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}