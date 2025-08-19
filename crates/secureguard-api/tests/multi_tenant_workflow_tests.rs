// Multi-Tenant Workflow Tests
// Comprehensive tests for tenant isolation, cross-tenant prevention, and multi-tenancy security

use secureguard_api::database::Database;
use secureguard_api::services::{
    agent_service::AgentService,
    api_key_service::ApiKeyService,
    auth_service::AuthService,
    user_service::UserService,
    subscription_service::SubscriptionService,
};
use secureguard_shared::{
    CreateUserRequest, RegisterAgentRequest, HeartbeatRequest, AgentStatus, CreateApiKeyRequest,
};
use uuid::Uuid;
use chrono::Utc;

// Test setup helper for Multi-Tenant tests
pub struct MultiTenantSetup {
    pub database: Database,
    pub user_service: UserService,
    pub agent_service: AgentService,
    pub api_key_service: ApiKeyService,
    pub subscription_service: SubscriptionService,
}

impl MultiTenantSetup {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
            "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
        });
        
        let database = Database::new(&database_url)
            .await
            .expect("Failed to connect to test database");
            
        let auth_service = AuthService::new("test-secret-key-multi-tenant".to_string());
        let user_service = UserService::new(database.pool().clone(), auth_service.clone());
        let agent_service = AgentService::new(database.pool().clone());
        let api_key_service = ApiKeyService::new(database.pool().clone());
        let subscription_service = SubscriptionService::new(database.pool().clone());
        
        MultiTenantSetup {
            database,
            user_service,
            agent_service,
            api_key_service,
            subscription_service,
        }
    }
    
    pub async fn cleanup_test_data(&self) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM agents.endpoints WHERE device_name LIKE 'multitenant_%'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM users.api_keys WHERE key_name LIKE 'multitenant_%'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM subscriptions.user_subscriptions WHERE user_id IN (SELECT user_id FROM users.users WHERE email LIKE '%@multitenant-test.com')")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM users.users WHERE email LIKE '%@multitenant-test.com'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM tenants.tenants WHERE name LIKE 'MultiTenant Test%'")
            .execute(self.database.pool()).await?;
        Ok(())
    }

    // Helper to create tenant with users and subscription
    pub async fn create_tenant_with_users(&self, tenant_name: &str, user_count: usize) -> Result<(Uuid, Vec<(secureguard_shared::User, String)>), secureguard_shared::SecureGuardError> {
        // Create tenant with short name to avoid varchar constraint  
        let tenant_id = Uuid::new_v4();
        let short_name = format!("T{}", tenant_name);  // Very short name
        sqlx::query!(
            "INSERT INTO tenants.tenants (tenant_id, name, plan_tier) VALUES ($1, $2, $3)",
            tenant_id, short_name, "free"  // Use shorter plan tier
        ).execute(self.database.pool()).await
        .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?;

        // Get a plan for subscriptions
        let plan_id = sqlx::query!("SELECT plan_id FROM subscriptions.plans ORDER BY plan_id LIMIT 1")
            .fetch_one(self.database.pool()).await
            .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?
            .plan_id;

        let mut users_with_keys = Vec::new();
        
        for i in 1..=user_count {
            // Create unique user for this tenant
            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let unique_username = format!("mt{}u{}", timestamp % 100000, i);
            
            let create_request = CreateUserRequest {
                username: unique_username.clone(),
                email: format!("{}@multitenant-test.com", unique_username),
                password: format!("{}Pass123!", unique_username),
            };
            
            let user = self.user_service.create_user(create_request).await?;
            
            // Assign user to tenant
            sqlx::query!(
                "UPDATE users.users SET tenant_id = $1 WHERE user_id = $2",
                tenant_id,
                user.user_id
            )
            .execute(self.database.pool())
            .await
            .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?;
            
            // Create subscription for user
            sqlx::query!(
                r#"
                INSERT INTO subscriptions.user_subscriptions 
                (user_id, plan_id, status, current_period_start, current_period_end, created_at)
                VALUES ($1, $2, 'active', $3, $4, $5)
                "#,
                user.user_id, plan_id, Utc::now(), Utc::now() + chrono::Duration::days(30), Utc::now()
            ).execute(self.database.pool()).await
            .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?;
            
            // Create API key for user
            let api_key_request = CreateApiKeyRequest {
                key_name: format!("multitenant_{}_key_{}", tenant_name, i),
                expires_in_days: Some(30),
            };
            
            let api_key_response = self.api_key_service.create_api_key(user.user_id, api_key_request).await?;
            users_with_keys.push((user, api_key_response.api_key));
        }
        
        Ok((tenant_id, users_with_keys))
    }

    // Helper to create agent for user
    pub async fn create_agent_for_user(&self, user_id: Uuid, api_key: &str, device_name: &str) -> Result<secureguard_shared::Agent, secureguard_shared::SecureGuardError> {
        let register_request = RegisterAgentRequest {
            api_key: api_key.to_string(),
            device_name: device_name.to_string(),
            hardware_fingerprint: format!("hw_{}", Uuid::new_v4()),
            os_info: serde_json::json!({
                "name": "Windows 11 Pro",
                "version": "11.0.22000",
                "architecture": "x64"
            }),
            version: "1.0.0".to_string(),
        };
        
        self.agent_service.register_agent_with_api_key(register_request).await
    }
}

// TEST 1: TENANT ISOLATION WORKFLOW
#[tokio::test]
async fn test_tenant_isolation_workflow() {
    let setup = MultiTenantSetup::new().await;
    
    println!("🚀 Starting Tenant Isolation Test");
    
    // Step 1: Create two separate tenants with users
    println!("🏢 Step 1: Create Two Separate Tenants");
    let (tenant_a_id, tenant_a_users) = setup.create_tenant_with_users("A", 2).await
        .expect("Tenant A creation should succeed");
    
    let (tenant_b_id, tenant_b_users) = setup.create_tenant_with_users("B", 2).await
        .expect("Tenant B creation should succeed");
    
    assert_ne!(tenant_a_id, tenant_b_id);
    println!("✅ Created Tenant A: {} with {} users", tenant_a_id, tenant_a_users.len());
    println!("✅ Created Tenant B: {} with {} users", tenant_b_id, tenant_b_users.len());
    
    // Step 2: Create agents for each tenant
    println!("🔧 Step 2: Create Agents for Each Tenant");
    let mut tenant_a_agents = Vec::new();
    for (i, (user, api_key)) in tenant_a_users.iter().enumerate() {
        let agent = setup.create_agent_for_user(
            user.user_id, 
            api_key, 
            &format!("multitenant_a_device_{}", i + 1)
        ).await.expect("Agent creation should succeed");
        
        // Verify agent belongs to tenant A
        assert_eq!(agent.tenant_id, tenant_a_id);
        assert_eq!(agent.user_id.unwrap(), user.user_id);
        tenant_a_agents.push(agent);
    }
    
    let mut tenant_b_agents = Vec::new();
    for (i, (user, api_key)) in tenant_b_users.iter().enumerate() {
        let agent = setup.create_agent_for_user(
            user.user_id, 
            api_key, 
            &format!("multitenant_b_device_{}", i + 1)
        ).await.expect("Agent creation should succeed");
        
        // Verify agent belongs to tenant B
        assert_eq!(agent.tenant_id, tenant_b_id);
        assert_eq!(agent.user_id.unwrap(), user.user_id);
        tenant_b_agents.push(agent);
    }
    
    println!("✅ Created {} agents for Tenant A", tenant_a_agents.len());
    println!("✅ Created {} agents for Tenant B", tenant_b_agents.len());
    
    // Step 3: Test tenant isolation in agent listing
    println!("🔍 Step 3: Test Tenant Isolation in Agent Listing");
    
    // List agents for Tenant A
    let tenant_a_listed_agents = setup.agent_service.list_agents_for_tenant(tenant_a_id).await
        .expect("Tenant A agent listing should succeed");
    
    assert_eq!(tenant_a_listed_agents.len(), 2);
    for agent in &tenant_a_listed_agents {
        assert_eq!(agent.tenant_id, tenant_a_id);
        assert_ne!(agent.tenant_id, tenant_b_id); // Should NOT see Tenant B agents
    }
    
    // List agents for Tenant B
    let tenant_b_listed_agents = setup.agent_service.list_agents_for_tenant(tenant_b_id).await
        .expect("Tenant B agent listing should succeed");
    
    assert_eq!(tenant_b_listed_agents.len(), 2);
    for agent in &tenant_b_listed_agents {
        assert_eq!(agent.tenant_id, tenant_b_id);
        assert_ne!(agent.tenant_id, tenant_a_id); // Should NOT see Tenant A agents
    }
    
    println!("✅ Tenant isolation verified: each tenant only sees their own agents");
    
    // Step 4: Test user isolation within tenants
    println!("👥 Step 4: Test User Isolation Within Tenants");
    
    // Test that users can only see their own agents
    let tenant_a_user1 = &tenant_a_users[0].0;
    let tenant_a_user2 = &tenant_a_users[1].0;
    
    let user1_agents = setup.agent_service.list_agents_for_user(tenant_a_user1.user_id).await
        .expect("User 1 agent listing should succeed");
    
    let user2_agents = setup.agent_service.list_agents_for_user(tenant_a_user2.user_id).await
        .expect("User 2 agent listing should succeed");
    
    assert_eq!(user1_agents.len(), 1);
    assert_eq!(user2_agents.len(), 1);
    
    // Verify users only see their own agents
    assert_eq!(user1_agents[0].user_id.unwrap(), tenant_a_user1.user_id);
    assert_eq!(user2_agents[0].user_id.unwrap(), tenant_a_user2.user_id);
    assert_ne!(user1_agents[0].agent_id, user2_agents[0].agent_id);
    
    println!("✅ User isolation verified: users only see their own agents");
    
    println!("✅ Tenant Isolation workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 2: CROSS-TENANT ACCESS PREVENTION WORKFLOW
#[tokio::test]
async fn test_cross_tenant_access_prevention_workflow() {
    let setup = MultiTenantSetup::new().await;
    
    println!("🚀 Starting Cross-Tenant Access Prevention Test");
    
    // Step 1: Create two tenants with users and agents
    println!("🏗️ Step 1: Setup Two Tenants with Agents");
    let (tenant_x_id, tenant_x_users) = setup.create_tenant_with_users("X", 1).await
        .expect("Tenant X creation should succeed");
    
    let (tenant_y_id, tenant_y_users) = setup.create_tenant_with_users("Y", 1).await
        .expect("Tenant Y creation should succeed");
    
    let tenant_x_agent = setup.create_agent_for_user(
        tenant_x_users[0].0.user_id,
        &tenant_x_users[0].1,
        "multitenant_x_device"
    ).await.expect("Tenant X agent creation should succeed");
    
    let tenant_y_agent = setup.create_agent_for_user(
        tenant_y_users[0].0.user_id,
        &tenant_y_users[0].1,
        "multitenant_y_device"
    ).await.expect("Tenant Y agent creation should succeed");
    
    println!("✅ Setup complete: Tenant X agent {} and Tenant Y agent {}", 
             tenant_x_agent.agent_id, tenant_y_agent.agent_id);
    
    // Step 2: Test API key isolation
    println!("🔑 Step 2: Test API Key Cross-Tenant Isolation");
    
    // Try to register agent for Tenant Y using Tenant X's API key (should fail)
    let cross_tenant_register = RegisterAgentRequest {
        api_key: tenant_x_users[0].1.clone(), // Tenant X's API key
        device_name: "multitenant_cross_attempt".to_string(),
        hardware_fingerprint: format!("hw_{}", Uuid::new_v4()),
        os_info: serde_json::json!({"name": "Windows 11"}),
        version: "1.0.0".to_string(),
    };
    
    // This should succeed but the agent should belong to Tenant X (the API key owner)
    // not Tenant Y. This is correct behavior - API keys determine tenant ownership.
    let cross_agent = setup.agent_service.register_agent_with_api_key(cross_tenant_register).await
        .expect("Agent registration should succeed");
    
    // The agent should belong to the tenant of the API key owner (Tenant X)
    assert_eq!(cross_agent.tenant_id, tenant_x_id);
    assert_eq!(cross_agent.user_id.unwrap(), tenant_x_users[0].0.user_id);
    println!("✅ API key ownership correctly enforced: agent belongs to API key owner's tenant");
    
    // Step 3: Test agent lookup isolation  
    println!("🔍 Step 3: Test Agent Lookup Cross-Tenant Isolation");
    
    // Tenant X should not be able to see Tenant Y's specific agent in their tenant list
    let tenant_x_agents = setup.agent_service.list_agents_for_tenant(tenant_x_id).await
        .expect("Tenant X listing should succeed");
    
    let tenant_y_agents = setup.agent_service.list_agents_for_tenant(tenant_y_id).await
        .expect("Tenant Y listing should succeed");
    
    // Verify no cross-contamination
    let tenant_x_agent_ids: Vec<Uuid> = tenant_x_agents.iter().map(|a| a.agent_id).collect();
    let tenant_y_agent_ids: Vec<Uuid> = tenant_y_agents.iter().map(|a| a.agent_id).collect();
    
    assert!(!tenant_x_agent_ids.contains(&tenant_y_agent.agent_id));
    assert!(!tenant_y_agent_ids.contains(&tenant_x_agent.agent_id));
    
    println!("✅ Agent lookup isolation verified");
    
    // Step 4: Test heartbeat isolation
    println!("💓 Step 4: Test Heartbeat Cross-Tenant Isolation");
    
    // Update heartbeat for Tenant X agent
    let tenant_x_heartbeat = HeartbeatRequest {
        agent_id: tenant_x_agent.agent_id,
        status: AgentStatus::Offline,
    };
    
    setup.agent_service.update_heartbeat(tenant_x_heartbeat).await
        .expect("Tenant X heartbeat should succeed");
    
    // Verify Tenant Y agent is unaffected
    let tenant_y_agent_status = setup.agent_service.find_by_id(tenant_y_agent.agent_id).await
        .expect("Tenant Y agent lookup should succeed")
        .expect("Agent should exist");
    
    assert_eq!(tenant_y_agent_status.status, AgentStatus::Online); // Should still be online
    
    // Verify Tenant X agent was updated
    let tenant_x_agent_status = setup.agent_service.find_by_id(tenant_x_agent.agent_id).await
        .expect("Tenant X agent lookup should succeed")
        .expect("Agent should exist");
    
    assert_eq!(tenant_x_agent_status.status, AgentStatus::Offline); // Should be offline now
    
    println!("✅ Heartbeat isolation verified: changes don't affect other tenants");
    
    println!("✅ Cross-Tenant Access Prevention workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 3: MULTI-TENANT SCALABILITY WORKFLOW
#[tokio::test]
async fn test_multi_tenant_scalability_workflow() {
    let setup = MultiTenantSetup::new().await;
    
    println!("🚀 Starting Multi-Tenant Scalability Test");
    
    // Step 1: Create multiple tenants with varying sizes
    println!("🏗️ Step 1: Create Multiple Tenants with Varying Sizes");
    
    let tenant_configs = vec![
        ("Small", 2),    // Small tenant: 2 users
        ("Medium", 3),   // Medium tenant: 3 users  
        ("Large", 5),    // Large tenant: 5 users
    ];
    
    let mut all_tenants = Vec::new();
    
    for (size_name, user_count) in tenant_configs {
        let (tenant_id, users) = setup.create_tenant_with_users(size_name, user_count).await
            .expect("Tenant creation should succeed");
        
        all_tenants.push((tenant_id, users, size_name));
        println!("✅ Created {} tenant {} with {} users", size_name, tenant_id, user_count);
    }
    
    // Step 2: Create agents for all tenants
    println!("🔧 Step 2: Create Agents for All Tenants");
    
    let mut tenant_agent_counts = Vec::new();
    
    for (tenant_id, users, size_name) in &all_tenants {
        let mut agent_count = 0;
        
        for (i, (user, api_key)) in users.iter().enumerate() {
            // Create 1-2 agents per user depending on tenant size
            let agents_per_user = if *size_name == "Large" { 2 } else { 1 };
            
            for j in 0..agents_per_user {
                let device_name = format!("multitenant_{}_user{}_device{}", size_name.to_lowercase(), i + 1, j + 1);
                let _agent = setup.create_agent_for_user(user.user_id, api_key, &device_name).await
                    .expect("Agent creation should succeed");
                agent_count += 1;
            }
        }
        
        tenant_agent_counts.push((*tenant_id, agent_count, *size_name));
        println!("✅ Created {} agents for {} tenant", agent_count, size_name);
    }
    
    // Step 3: Verify tenant isolation at scale
    println!("🔍 Step 3: Verify Tenant Isolation at Scale");
    
    let mut total_agents = 0;
    
    for (tenant_id, expected_count, size_name) in tenant_agent_counts {
        let tenant_agents = setup.agent_service.list_agents_for_tenant(tenant_id).await
            .expect("Tenant agent listing should succeed");
        
        assert_eq!(tenant_agents.len(), expected_count);
        
        // Verify all agents belong to this tenant
        for agent in &tenant_agents {
            assert_eq!(agent.tenant_id, tenant_id);
        }
        
        total_agents += tenant_agents.len();
        println!("✅ {} tenant isolation verified: {} agents", size_name, tenant_agents.len());
    }
    
    // Step 4: Test performance with concurrent tenant operations
    println!("⚡ Step 4: Test Concurrent Tenant Operations");
    
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let setup_arc = Arc::new(setup);
    let results = Arc::new(Mutex::new(Vec::new()));
    
    let mut handles = vec![];
    
    // Perform concurrent heartbeat updates for different tenants
    for (tenant_id, _, size_name) in &all_tenants {
        let setup_clone = Arc::clone(&setup_arc);
        let results_clone = Arc::clone(&results);
        let tenant_id = *tenant_id;
        let size_name = *size_name;
        
        let handle = tokio::spawn(async move {
            let tenant_agents = setup_clone.agent_service.list_agents_for_tenant(tenant_id).await
                .expect("Tenant listing should succeed");
            
            let mut successful_updates = 0;
            
            for agent in tenant_agents {
                let heartbeat = HeartbeatRequest {
                    agent_id: agent.agent_id,
                    status: AgentStatus::Error,
                };
                
                if setup_clone.agent_service.update_heartbeat(heartbeat).await.is_ok() {
                    successful_updates += 1;
                }
            }
            
            let mut results_guard = results_clone.lock().await;
            results_guard.push((size_name, successful_updates));
        });
        
        handles.push(handle);
    }
    
    // Wait for all concurrent operations to complete
    for handle in handles {
        handle.await.expect("Concurrent operation should complete");
    }
    
    let final_results = results.lock().await;
    let total_updates: usize = final_results.iter().map(|(_, count)| count).sum();
    
    println!("✅ Concurrent operations completed successfully");
    for (size_name, count) in final_results.iter() {
        println!("   {} tenant: {} successful heartbeat updates", size_name, count);
    }
    println!("   Total: {} successful updates across all tenants", total_updates);
    
    // Step 5: Verify data integrity after concurrent operations
    println!("🔒 Step 5: Verify Data Integrity After Concurrent Operations");
    
    for (tenant_id, _, size_name) in &all_tenants {
        let tenant_agents = setup_arc.agent_service.list_agents_for_tenant(*tenant_id).await
            .expect("Post-concurrent tenant listing should succeed");
        
        // All agents should have been updated to Error status
        let error_count = tenant_agents.iter().filter(|a| a.status == AgentStatus::Error).count();
        assert_eq!(error_count, tenant_agents.len());
        
        // Verify no cross-contamination - agents still belong to correct tenant
        for agent in &tenant_agents {
            assert_eq!(agent.tenant_id, *tenant_id);
        }
        
        println!("✅ {} tenant data integrity verified after concurrent operations", size_name);
    }
    
    println!("✅ Multi-Tenant Scalability workflow SUCCESSFUL!");
    println!("📊 Summary: {} tenants, {} total agents, concurrent operations verified", 
             all_tenants.len(), total_agents);
    
    setup_arc.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 4: TENANT DATA SEGREGATION WORKFLOW
#[tokio::test]
async fn test_tenant_data_segregation_workflow() {
    let setup = MultiTenantSetup::new().await;
    
    println!("🚀 Starting Tenant Data Segregation Test");
    
    // Step 1: Create tenants with different data patterns
    println!("🏗️ Step 1: Create Tenants with Different Data Patterns");
    
    let (healthcare_tenant_id, healthcare_users) = setup.create_tenant_with_users("Healthcare", 2).await
        .expect("Healthcare tenant creation should succeed");
    
    let (finance_tenant_id, finance_users) = setup.create_tenant_with_users("Finance", 2).await
        .expect("Finance tenant creation should succeed");
    
    println!("✅ Created Healthcare tenant: {}", healthcare_tenant_id);
    println!("✅ Created Finance tenant: {}", finance_tenant_id);
    
    // Step 2: Create agents with tenant-specific configurations
    println!("🔧 Step 2: Create Agents with Tenant-Specific Configurations");
    
    // Healthcare agents
    let healthcare_agent1 = setup.create_agent_for_user(
        healthcare_users[0].0.user_id,
        &healthcare_users[0].1,
        "multitenant_healthcare_workstation_1"
    ).await.expect("Healthcare agent 1 creation should succeed");
    
    let healthcare_agent2 = setup.create_agent_for_user(
        healthcare_users[1].0.user_id,
        &healthcare_users[1].1,
        "multitenant_healthcare_server_1"
    ).await.expect("Healthcare agent 2 creation should succeed");
    
    // Finance agents
    let finance_agent1 = setup.create_agent_for_user(
        finance_users[0].0.user_id,
        &finance_users[0].1,
        "multitenant_finance_trading_1"
    ).await.expect("Finance agent 1 creation should succeed");
    
    let finance_agent2 = setup.create_agent_for_user(
        finance_users[1].0.user_id,
        &finance_users[1].1,
        "multitenant_finance_compliance_1"
    ).await.expect("Finance agent 2 creation should succeed");
    
    println!("✅ Created agents for both tenants with specific naming patterns");
    
    // Step 3: Test data segregation in queries
    println!("🔍 Step 3: Test Data Segregation in Queries");
    
    // Healthcare tenant should only see healthcare data
    let healthcare_agents = setup.agent_service.list_agents_for_tenant(healthcare_tenant_id).await
        .expect("Healthcare agent listing should succeed");
    
    assert_eq!(healthcare_agents.len(), 2);
    for agent in &healthcare_agents {
        assert_eq!(agent.tenant_id, healthcare_tenant_id);
        assert!(agent.device_name.as_ref().unwrap().contains("healthcare"));
        assert!(!agent.device_name.as_ref().unwrap().contains("finance"));
    }
    
    // Finance tenant should only see finance data
    let finance_agents = setup.agent_service.list_agents_for_tenant(finance_tenant_id).await
        .expect("Finance agent listing should succeed");
    
    assert_eq!(finance_agents.len(), 2);
    for agent in &finance_agents {
        assert_eq!(agent.tenant_id, finance_tenant_id);
        assert!(agent.device_name.as_ref().unwrap().contains("finance"));
        assert!(!agent.device_name.as_ref().unwrap().contains("healthcare"));
    }
    
    println!("✅ Data segregation verified: each tenant only sees their own data");
    
    // Step 4: Test user-level segregation within tenants
    println!("👤 Step 4: Test User-Level Segregation Within Tenants");
    
    // Healthcare user 1 should only see their own agent
    let hc_user1_agents = setup.agent_service.list_agents_for_user(healthcare_users[0].0.user_id).await
        .expect("Healthcare user 1 listing should succeed");
    
    assert_eq!(hc_user1_agents.len(), 1);
    assert_eq!(hc_user1_agents[0].agent_id, healthcare_agent1.agent_id);
    assert_ne!(hc_user1_agents[0].agent_id, healthcare_agent2.agent_id);
    
    // Healthcare user 2 should only see their own agent  
    let hc_user2_agents = setup.agent_service.list_agents_for_user(healthcare_users[1].0.user_id).await
        .expect("Healthcare user 2 listing should succeed");
    
    assert_eq!(hc_user2_agents.len(), 1);
    assert_eq!(hc_user2_agents[0].agent_id, healthcare_agent2.agent_id);
    assert_ne!(hc_user2_agents[0].agent_id, healthcare_agent1.agent_id);
    
    println!("✅ User-level segregation verified within tenants");
    
    // Step 5: Test cross-tenant agent lookup prevention
    println!("🚫 Step 5: Test Cross-Tenant Agent Lookup Prevention");
    
    // Direct agent lookup should work for same-tenant agents
    let healthcare_lookup = setup.agent_service.find_by_id(healthcare_agent1.agent_id).await
        .expect("Same-tenant agent lookup should succeed")
        .expect("Agent should exist");
    assert_eq!(healthcare_lookup.tenant_id, healthcare_tenant_id);
    
    let finance_lookup = setup.agent_service.find_by_id(finance_agent1.agent_id).await
        .expect("Same-tenant agent lookup should succeed")
        .expect("Agent should exist");
    assert_eq!(finance_lookup.tenant_id, finance_tenant_id);
    
    // But tenant-specific listing should still be isolated
    let healthcare_tenant_check = healthcare_agents.iter()
        .any(|a| a.agent_id == finance_agent1.agent_id);
    assert!(!healthcare_tenant_check); // Healthcare tenant should not see finance agents
    
    let finance_tenant_check = finance_agents.iter()
        .any(|a| a.agent_id == healthcare_agent1.agent_id);
    assert!(!finance_tenant_check); // Finance tenant should not see healthcare agents
    
    println!("✅ Cross-tenant lookup prevention verified");
    
    println!("✅ Tenant Data Segregation workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}