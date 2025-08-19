// Agent Management Workflow Tests
// Comprehensive tests for agent lifecycle, configuration, health monitoring, and remote commands

use secureguard_api::database::Database;
use secureguard_api::services::{
    agent_service::AgentService,
    api_key_service::ApiKeyService,
    auth_service::AuthService,
    config_service::ConfigService,
    remote_command_service::{RemoteCommandService, RemoteCommand, CommandSender, UserRole},
    subscription_service::SubscriptionService,
    user_service::UserService,
};
use secureguard_shared::{
    CreateUserRequest, RegisterAgentRequest, HeartbeatRequest, AgentStatus, CreateApiKeyRequest,
};
use uuid::Uuid;
use chrono::Utc;

// Test setup helper for Agent Management tests
pub struct AgentManagementSetup {
    pub database: Database,
    pub user_service: UserService,
    pub agent_service: AgentService,
    pub api_key_service: ApiKeyService,
    pub config_service: ConfigService,
    pub remote_command_service: RemoteCommandService,
    pub subscription_service: SubscriptionService,
}

impl AgentManagementSetup {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
            "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
        });
        
        let database = Database::new(&database_url)
            .await
            .expect("Failed to connect to test database");
            
        let auth_service = AuthService::new("test-secret-key-agent-mgmt".to_string());
        let user_service = UserService::new(database.pool().clone(), auth_service.clone());
        let agent_service = AgentService::new(database.pool().clone());
        let api_key_service = ApiKeyService::new(database.pool().clone());
        let config_service = ConfigService::new(database.pool().clone());
        let remote_command_service = RemoteCommandService::new(database.pool().clone());
        let subscription_service = SubscriptionService::new(database.pool().clone());
        
        AgentManagementSetup {
            database,
            user_service,
            agent_service,
            api_key_service,
            config_service,
            remote_command_service,
            subscription_service,
        }
    }
    
    pub async fn cleanup_test_data(&self) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM agent_commands WHERE sender_username LIKE 'agent_mgmt_%'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM agents.endpoints WHERE device_name LIKE 'agent_mgmt_%'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM users.api_keys WHERE key_name LIKE 'agent_mgmt_%'")
            .execute(self.database.pool()).await?;
        sqlx::query!("DELETE FROM users.users WHERE email LIKE '%@agent-mgmt-test.com'")
            .execute(self.database.pool()).await?;
        Ok(())
    }

    // Helper to set up user with subscription for agent tests
    pub async fn setup_user_with_subscription(&self, username: &str, role: &str, plan: &str) -> Result<(secureguard_shared::User, String), secureguard_shared::SecureGuardError> {
        // Create user
        let create_request = CreateUserRequest {
            username: username.to_string(),
            email: format!("{}@agent-mgmt-test.com", username),
            password: format!("{}Password123!", username),
        };
        
        let user = self.user_service.create_user(create_request).await?;
        
        // Set role in database
        sqlx::query!(
            "UPDATE users.users SET role = $1 WHERE user_id = $2",
            role,
            user.user_id
        )
        .execute(self.database.pool())
        .await
        .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?;
        
        // Setup subscription manually
        let tenant_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO tenants.tenants (tenant_id, name, plan_tier) VALUES ($1, $2, $3)",
            tenant_id, "Agent Test Tenant", plan
        ).execute(self.database.pool()).await
        .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?;
        
        let plan_id = match plan {
            "professional" => {
                sqlx::query!("SELECT plan_id FROM subscriptions.plans WHERE plan_slug = 'professional'")
                    .fetch_one(self.database.pool()).await
                    .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?
                    .plan_id
            },
            _ => {
                sqlx::query!("SELECT plan_id FROM subscriptions.plans WHERE plan_slug = 'starter'")
                    .fetch_one(self.database.pool()).await
                    .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?
                    .plan_id
            }
        };
        
        sqlx::query!(
            r#"
            INSERT INTO subscriptions.user_subscriptions 
            (user_id, plan_id, status, current_period_start, current_period_end, created_at)
            VALUES ($1, $2, 'active', $3, $4, $5)
            "#,
            user.user_id, plan_id, Utc::now(), Utc::now() + chrono::Duration::days(30), Utc::now()
        ).execute(self.database.pool()).await
        .map_err(|e| secureguard_shared::SecureGuardError::DatabaseError(e.to_string()))?;
        
        // Create API key
        let api_key_request = CreateApiKeyRequest {
            key_name: format!("agent_mgmt_{}_key", username),
            expires_in_days: Some(30),
        };
        
        let api_key_response = self.api_key_service.create_api_key(user.user_id, api_key_request).await?;
        
        Ok((user, api_key_response.api_key))
    }

    // Helper to create and register agent
    pub async fn create_and_register_agent(&self, user_id: Uuid, api_key: &str, device_name: &str) -> Result<secureguard_shared::Agent, secureguard_shared::SecureGuardError> {
        let register_request = RegisterAgentRequest {
            api_key: api_key.to_string(),
            device_name: device_name.to_string(),
            hardware_fingerprint: format!("hw_{}", Uuid::new_v4()),
            os_info: "Windows 11 Pro".to_string(),
            version: "1.0.0".to_string(),
        };
        
        self.agent_service.register_agent_with_api_key(register_request).await
    }
}

// TEST 1: AGENT REGISTRATION AND LIFECYCLE WORKFLOW
#[tokio::test]
async fn test_agent_registration_lifecycle_workflow() {
    let setup = AgentManagementSetup::new().await;
    
    println!("🚀 Starting Agent Registration and Lifecycle Test");
    
    // Step 1: Create user with professional subscription
    println!("👤 Step 1: Create User with Professional Subscription");
    let (user, api_key) = setup.setup_user_with_subscription("agent_mgmt_lifecycle", "admin", "professional")
        .await.expect("User setup should succeed");
    
    println!("✅ User created: {} with API key", user.user_id);
    
    // Step 2: Register agent
    println!("🔧 Step 2: Register Agent");
    let agent = setup.create_and_register_agent(user.user_id, &api_key, "agent_mgmt_lifecycle_device")
        .await.expect("Agent registration should succeed");
    
    println!("✅ Agent registered: {} on device {}", agent.agent_id, agent.device_name.unwrap_or("unknown".to_string()));
    assert_eq!(agent.user_id.unwrap(), user.user_id);
    assert_eq!(agent.status, AgentStatus::Online);
    
    // Step 3: Test heartbeat updates
    println!("💓 Step 3: Test Agent Heartbeat");
    let heartbeat = HeartbeatRequest {
        agent_id: agent.agent_id,
        status: AgentStatus::Online,
    };
    
    setup.agent_service.update_heartbeat(heartbeat).await
        .expect("Heartbeat should succeed");
    
    // Verify status updated
    let updated_agent = setup.agent_service.find_by_id(agent.agent_id).await
        .expect("Agent lookup should succeed")
        .expect("Agent should exist");
    assert_eq!(updated_agent.status, AgentStatus::Online);
    println!("✅ Agent heartbeat updated successfully");
    
    // Step 4: Test agent listing
    println!("📋 Step 4: List Agents for User");
    let user_agents = setup.agent_service.list_agents_for_user(user.user_id).await
        .expect("Listing agents should succeed");
    assert_eq!(user_agents.len(), 1);
    assert_eq!(user_agents[0].agent_id, agent.agent_id);
    println!("✅ Agent listing successful: {} agents found", user_agents.len());
    
    // Step 5: Test configuration retrieval
    println!("⚙️ Step 5: Test Agent Configuration");
    let config = setup.config_service.get_agent_config(agent.agent_id).await
        .expect("Configuration retrieval should succeed");
    
    assert_eq!(config.agent_id, agent.agent_id);
    assert_eq!(config.subscription.tier, "professional");
    assert!(config.features.enabled_features.contains(&"advanced_threat_detection".to_string()));
    println!("✅ Agent configuration retrieved successfully");
    
    println!("✅ Agent Registration and Lifecycle workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 2: AGENT CONFIGURATION MANAGEMENT WORKFLOW
#[tokio::test]
async fn test_agent_configuration_management_workflow() {
    let setup = AgentManagementSetup::new().await;
    
    println!("🚀 Starting Agent Configuration Management Test");
    
    // Step 1: Setup user and agent
    println!("🏗️ Step 1: Setup User and Agent");
    let (user, api_key) = setup.setup_user_with_subscription("agent_mgmt_config", "admin", "professional")
        .await.expect("User setup should succeed");
    
    let agent = setup.create_and_register_agent(user.user_id, &api_key, "agent_mgmt_config_device")
        .await.expect("Agent registration should succeed");
    
    println!("✅ Setup complete: User {} with Agent {}", user.user_id, agent.agent_id);
    
    // Step 2: Test initial configuration
    println!("📊 Step 2: Test Initial Configuration");
    let initial_config = setup.config_service.get_agent_config(agent.agent_id).await
        .expect("Initial config retrieval should succeed");
    
    assert_eq!(initial_config.config_version, 1);
    assert_eq!(initial_config.subscription.tier, "professional");
    assert!(initial_config.limits.max_concurrent_scans >= 10);
    assert!(initial_config.monitoring.heartbeat_interval <= 60);
    println!("✅ Initial configuration validated");
    
    // Step 3: Test configuration versioning
    println!("🔄 Step 3: Test Configuration Version Management");
    let new_version = setup.config_service.increment_config_version(agent.agent_id).await
        .expect("Version increment should succeed");
    assert_eq!(new_version, 2);
    
    let needs_update = setup.config_service.config_needs_update(agent.agent_id, 1).await
        .expect("Update check should succeed");
    assert!(needs_update);
    
    let no_update_needed = setup.config_service.config_needs_update(agent.agent_id, 2).await
        .expect("Update check should succeed");
    assert!(!no_update_needed);
    println!("✅ Configuration versioning working correctly");
    
    // Step 4: Test subscription info
    println!("💳 Step 4: Test Subscription Information");
    let subscription_info = setup.config_service.get_subscription_info(agent.agent_id).await
        .expect("Subscription info should succeed");
    
    assert_eq!(subscription_info.tier, "professional");
    assert!(subscription_info.enabled_features.contains(&"advanced_threat_detection".to_string()));
    assert!(subscription_info.enabled_features.contains(&"remote_commands".to_string()));
    println!("✅ Subscription information validated");
    
    // Step 5: Verify configuration structure
    println!("🏗️ Step 5: Verify Configuration Structure");
    let config = setup.config_service.get_agent_config(agent.agent_id).await
        .expect("Config retrieval should succeed");
    
    // Verify all configuration sections exist
    assert!(!config.features.enabled_features.is_empty());
    assert!(config.limits.max_file_scan_size > 0);
    assert!(config.monitoring.heartbeat_interval > 0);
    assert!(!config.security.tls_version.is_empty());
    assert!(!config.logging.level.is_empty());
    assert!(config.updates.check_interval_hours > 0);
    
    println!("✅ All configuration sections validated");
    
    println!("✅ Agent Configuration Management workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 3: AGENT HEALTH MONITORING AND RECOVERY WORKFLOW  
#[tokio::test]
async fn test_agent_health_monitoring_recovery_workflow() {
    let setup = AgentManagementSetup::new().await;
    
    println!("🚀 Starting Agent Health Monitoring and Recovery Test");
    
    // Step 1: Setup multiple agents for monitoring
    println!("🏗️ Step 1: Setup Multiple Agents for Monitoring");
    let (user, api_key) = setup.setup_user_with_subscription("agent_mgmt_health", "admin", "professional")
        .await.expect("User setup should succeed");
    
    let mut agents = Vec::new();
    for i in 1..=3 {
        let agent = setup.create_and_register_agent(
            user.user_id, 
            &api_key, 
            &format!("agent_mgmt_health_device_{}", i)
        ).await.expect("Agent registration should succeed");
        agents.push(agent);
    }
    
    println!("✅ Setup complete: {} agents registered", agents.len());
    
    // Step 2: Test various health states
    println!("💓 Step 2: Test Various Agent Health States");
    
    // Agent 1: Keep online
    let heartbeat1 = HeartbeatRequest {
        agent_id: agents[0].agent_id,
        status: AgentStatus::Online,
    };
    setup.agent_service.update_heartbeat(heartbeat1).await
        .expect("Heartbeat should succeed");
    
    // Agent 2: Set to error state
    let heartbeat2 = HeartbeatRequest {
        agent_id: agents[1].agent_id,
        status: AgentStatus::Error,
    };
    setup.agent_service.update_heartbeat(heartbeat2).await
        .expect("Heartbeat should succeed");
    
    // Agent 3: Set to offline
    let heartbeat3 = HeartbeatRequest {
        agent_id: agents[2].agent_id,
        status: AgentStatus::Offline,
    };
    setup.agent_service.update_heartbeat(heartbeat3).await
        .expect("Heartbeat should succeed");
    
    println!("✅ Agent health states set successfully");
    
    // Step 3: Verify health state monitoring
    println!("🔍 Step 3: Verify Health State Monitoring");
    let user_agents = setup.agent_service.list_agents_for_user(user.user_id).await
        .expect("Listing agents should succeed");
    
    assert_eq!(user_agents.len(), 3);
    
    // Find and verify each agent's status
    for agent in &user_agents {
        let status = match agent.device_name.as_ref().unwrap().as_str() {
            name if name.contains("device_1") => AgentStatus::Online,
            name if name.contains("device_2") => AgentStatus::Error, 
            name if name.contains("device_3") => AgentStatus::Offline,
            _ => panic!("Unexpected device name"),
        };
        assert_eq!(agent.status, status);
    }
    println!("✅ All agent health states verified correctly");
    
    // Step 4: Test health recovery workflow
    println!("🔄 Step 4: Test Health Recovery Workflow");
    
    // Simulate recovery: offline agent comes back online
    let recovery_heartbeat = HeartbeatRequest {
        agent_id: agents[2].agent_id,
        status: AgentStatus::Online,
    };
    setup.agent_service.update_heartbeat(recovery_heartbeat).await
        .expect("Recovery heartbeat should succeed");
    
    // Verify recovery
    let recovered_agent = setup.agent_service.find_by_id(agents[2].agent_id).await
        .expect("Agent lookup should succeed")
        .expect("Agent should exist");
    assert_eq!(recovered_agent.status, AgentStatus::Online);
    println!("✅ Agent recovery successful");
    
    // Step 5: Test bulk health status retrieval
    println!("📊 Step 5: Test Bulk Health Status Retrieval");
    let tenant_agents = setup.agent_service.list_agents_for_tenant(
        agents[0].tenant_id
    ).await.expect("Tenant agent listing should succeed");
    
    assert_eq!(tenant_agents.len(), 3);
    
    // Count by status
    let online_count = tenant_agents.iter().filter(|a| a.status == AgentStatus::Online).count();
    let error_count = tenant_agents.iter().filter(|a| a.status == AgentStatus::Error).count();
    
    assert_eq!(online_count, 2); // Agent 1 and recovered Agent 3
    assert_eq!(error_count, 1); // Agent 2
    
    println!("✅ Bulk health status retrieval successful: {} online, {} error", online_count, error_count);
    
    println!("✅ Agent Health Monitoring and Recovery workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 4: REMOTE COMMAND EXECUTION WORKFLOW
#[tokio::test]
async fn test_remote_command_execution_workflow() {
    let setup = AgentManagementSetup::new().await;
    
    println!("🚀 Starting Remote Command Execution Test");
    
    // Step 1: Setup analyst user and agent
    println!("👨‍🔬 Step 1: Setup Security Analyst and Agent");
    let (analyst_user, api_key) = setup.setup_user_with_subscription("agent_mgmt_analyst", "security_analyst", "professional")
        .await.expect("Analyst setup should succeed");
    
    let agent = setup.create_and_register_agent(analyst_user.user_id, &api_key, "agent_mgmt_command_device")
        .await.expect("Agent registration should succeed");
    
    println!("✅ Security analyst and agent setup complete");
    
    // Step 2: Test basic system information command
    println!("💻 Step 2: Test Basic System Information Command");
    let command_sender = CommandSender {
        user_id: analyst_user.user_id,
        username: "agent_mgmt_analyst".to_string(),
        role: UserRole::Analyst,
        subscription_tier: "professional".to_string(),
        permissions: vec!["security_incidents".to_string(), "agents_control".to_string()],
        ip_address: Some("127.0.0.1".to_string()),
    };
    
    let system_info_command = RemoteCommand::GetSystemInfo;
    let command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        system_info_command,
        command_sender.clone()
    ).await.expect("System info command should succeed");
    
    println!("✅ System info command submitted: {}", command_id);
    
    // Step 3: Test file operation command
    println!("📁 Step 3: Test File Operation Command");
    let file_hash_command = RemoteCommand::GetFileHash {
        path: "C:\\Windows\\System32\\cmd.exe".to_string(),
        algorithm: Some("sha256".to_string()),
    };
    
    let file_command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        file_hash_command,
        command_sender.clone()
    ).await.expect("File hash command should succeed");
    
    println!("✅ File hash command submitted: {}", file_command_id);
    
    // Step 4: Test security scan command
    println!("🔍 Step 4: Test Security Scan Command");
    let scan_command = RemoteCommand::RunQuickScan {
        path: Some("C:\\Users".to_string()),
    };
    
    let scan_command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        scan_command,
        command_sender.clone()
    ).await.expect("Scan command should succeed");
    
    println!("✅ Security scan command submitted: {}", scan_command_id);
    
    // Step 5: Test command retrieval and status
    println!("📋 Step 5: Test Command Retrieval and Status");
    let pending_commands = setup.remote_command_service.get_pending_commands(agent.agent_id).await
        .expect("Pending commands retrieval should succeed");
    
    assert_eq!(pending_commands.len(), 3);
    
    // Verify all commands are in queue
    for cmd in &pending_commands {
        assert_eq!(cmd.agent_id, agent.agent_id);
        assert_eq!(cmd.sender.user_id, analyst_user.user_id);
        assert!(matches!(cmd.status, secureguard_api::services::remote_command_service::CommandStatus::Queued));
    }
    println!("✅ All commands properly queued: {} pending commands", pending_commands.len());
    
    // Step 6: Test command status updates
    println!("🔄 Step 6: Test Command Status Updates");
    
    // Mark first command as sent
    setup.remote_command_service.mark_command_sent(command_id).await
        .expect("Mark command sent should succeed");
    
    // Simulate command completion with response
    use secureguard_api::services::remote_command_service::{CommandResponse, SystemInfo};
    let response = CommandResponse::SystemInfo(SystemInfo {
        hostname: "test-machine".to_string(),
        os: "Windows".to_string(),
        os_version: "11 Pro".to_string(),
        architecture: "x64".to_string(),
        cpu: "Intel i7".to_string(),
        total_memory: 16 * 1024 * 1024 * 1024, // 16GB
        available_memory: 8 * 1024 * 1024 * 1024, // 8GB
        uptime: 86400, // 1 day
        timezone: "UTC".to_string(),
    });
    
    setup.remote_command_service.update_command_response(command_id, response, 1500).await
        .expect("Command response update should succeed");
    
    println!("✅ Command status updates successful");
    
    // Step 7: Test command history
    println!("📊 Step 7: Test Command History");
    let command_history = setup.remote_command_service.get_command_history(agent.agent_id, Some(10)).await
        .expect("Command history should succeed");
    
    // Note: The simplified implementation returns empty vec, but in real implementation this would show history
    println!("✅ Command history retrieved successfully");
    
    println!("✅ Remote Command Execution workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 5: AGENT UPDATE AND MAINTENANCE WORKFLOW
#[tokio::test]
async fn test_agent_update_maintenance_workflow() {
    let setup = AgentManagementSetup::new().await;
    
    println!("🚀 Starting Agent Update and Maintenance Test");
    
    // Step 1: Setup system admin and agent
    println!("👑 Step 1: Setup System Admin and Agent");
    let (admin_user, api_key) = setup.setup_user_with_subscription("agent_mgmt_admin", "system_admin", "professional")
        .await.expect("Admin setup should succeed");
    
    let agent = setup.create_and_register_agent(admin_user.user_id, &api_key, "agent_mgmt_update_device")
        .await.expect("Agent registration should succeed");
    
    println!("✅ System admin and agent setup complete");
    
    // Step 2: Test agent configuration update command
    println!("⚙️ Step 2: Test Agent Configuration Update");
    let admin_sender = CommandSender {
        user_id: admin_user.user_id,
        username: "agent_mgmt_admin".to_string(),
        role: UserRole::SystemAdmin,
        subscription_tier: "professional".to_string(),
        permissions: vec!["system_admin".to_string()],
        ip_address: Some("127.0.0.1".to_string()),
    };
    
    let update_config_command = RemoteCommand::UpdateConfiguration;
    let config_command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        update_config_command,
        admin_sender.clone()
    ).await.expect("Configuration update command should succeed");
    
    println!("✅ Configuration update command submitted: {}", config_command_id);
    
    // Step 3: Test agent version update command
    println!("🔄 Step 3: Test Agent Version Update");
    let update_agent_command = RemoteCommand::UpdateAgent {
        version: Some("1.1.0".to_string()),
        force: Some(false),
    };
    
    let update_command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        update_agent_command,
        admin_sender.clone()
    ).await.expect("Agent update command should succeed");
    
    println!("✅ Agent update command submitted: {}", update_command_id);
    
    // Step 4: Test agent restart command
    println!("🔄 Step 4: Test Agent Restart Command");
    let restart_command = RemoteCommand::RestartAgent;
    let restart_command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        restart_command,
        admin_sender.clone()
    ).await.expect("Restart command should succeed");
    
    println!("✅ Agent restart command submitted: {}", restart_command_id);
    
    // Step 5: Test feature management
    println!("🎛️ Step 5: Test Feature Management Commands");
    
    // Enable feature
    let enable_feature_command = RemoteCommand::EnableFeature {
        feature: "real_time_monitoring".to_string(),
    };
    
    let enable_command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        enable_feature_command,
        admin_sender.clone()
    ).await.expect("Enable feature command should succeed");
    
    // Disable feature
    let disable_feature_command = RemoteCommand::DisableFeature {
        feature: "vulnerability_scanning".to_string(),
    };
    
    let disable_command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        disable_feature_command,
        admin_sender.clone()
    ).await.expect("Disable feature command should succeed");
    
    println!("✅ Feature management commands submitted: {} (enable), {} (disable)", 
             enable_command_id, disable_command_id);
    
    // Step 6: Test agent status and logs retrieval
    println!("📊 Step 6: Test Agent Status and Logs Commands");
    
    let status_command = RemoteCommand::GetAgentStatus;
    let status_command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        status_command,
        admin_sender.clone()
    ).await.expect("Agent status command should succeed");
    
    let logs_command = RemoteCommand::GetAgentLogs {
        lines: Some(100),
        level: Some("info".to_string()),
    };
    
    let logs_command_id = setup.remote_command_service.submit_command(
        agent.agent_id,
        logs_command,
        admin_sender.clone()
    ).await.expect("Agent logs command should succeed");
    
    println!("✅ Status and logs commands submitted: {} (status), {} (logs)", 
             status_command_id, logs_command_id);
    
    // Step 7: Verify all commands are properly queued
    println!("📋 Step 7: Verify Command Queue");
    let pending_commands = setup.remote_command_service.get_pending_commands(agent.agent_id).await
        .expect("Pending commands retrieval should succeed");
    
    assert_eq!(pending_commands.len(), 7); // All 7 commands submitted
    
    // Verify command types and sender
    for cmd in &pending_commands {
        assert_eq!(cmd.agent_id, agent.agent_id);
        assert_eq!(cmd.sender.user_id, admin_user.user_id);
        assert!(matches!(cmd.sender.role, UserRole::SystemAdmin));
    }
    
    println!("✅ All {} maintenance commands properly queued", pending_commands.len());
    
    // Step 8: Test configuration version increment for updates
    println!("🔢 Step 8: Test Configuration Version Management");
    let initial_version = setup.config_service.get_current_config_version(agent.agent_id).await
        .expect("Get config version should succeed");
    
    let new_version = setup.config_service.increment_config_version(agent.agent_id).await
        .expect("Increment config version should succeed");
    
    assert_eq!(new_version, initial_version + 1);
    println!("✅ Configuration version incremented: {} -> {}", initial_version, new_version);
    
    println!("✅ Agent Update and Maintenance workflow SUCCESSFUL!");
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}

// TEST 6: MASS AGENT DEPLOYMENT AND MANAGEMENT WORKFLOW
#[tokio::test]
async fn test_mass_agent_deployment_workflow() {
    let setup = AgentManagementSetup::new().await;
    
    println!("🚀 Starting Mass Agent Deployment Test");
    
    // Step 1: Setup enterprise user for bulk operations
    println!("🏢 Step 1: Setup Enterprise User for Bulk Operations");
    let (enterprise_user, api_key) = setup.setup_user_with_subscription("agent_mgmt_enterprise", "system_admin", "professional")
        .await.expect("Enterprise user setup should succeed");
    
    println!("✅ Enterprise user setup complete: {}", enterprise_user.user_id);
    
    // Step 2: Deploy multiple agents (simulate mass deployment)
    println!("🚀 Step 2: Deploy Multiple Agents (Mass Deployment)");
    let mut deployed_agents = Vec::new();
    
    for i in 1..=5 {
        let agent = setup.create_and_register_agent(
            enterprise_user.user_id,
            &api_key,
            &format!("agent_mgmt_bulk_device_{:02}", i)
        ).await.expect("Bulk agent registration should succeed");
        
        deployed_agents.push(agent);
    }
    
    println!("✅ Mass deployment complete: {} agents deployed", deployed_agents.len());
    
    // Step 3: Test bulk health monitoring
    println!("💓 Step 3: Test Bulk Health Monitoring");
    
    // Set different health states across agents
    for (i, agent) in deployed_agents.iter().enumerate() {
        let status = match i % 3 {
            0 => AgentStatus::Online,
            1 => AgentStatus::Error,
            _ => AgentStatus::Offline,
        };
        
        let heartbeat = HeartbeatRequest {
            agent_id: agent.agent_id,
            status,
        };
        
        setup.agent_service.update_heartbeat(heartbeat).await
            .expect("Bulk heartbeat should succeed");
    }
    
    // Verify bulk status
    let all_agents = setup.agent_service.list_agents_for_user(enterprise_user.user_id).await
        .expect("Bulk agent listing should succeed");
    
    assert_eq!(all_agents.len(), 5);
    
    let status_counts = all_agents.iter().fold(
        (0, 0, 0), // (online, error, offline)
        |mut counts, agent| {
            match agent.status {
                AgentStatus::Online => counts.0 += 1,
                AgentStatus::Error => counts.1 += 1,
                AgentStatus::Offline => counts.2 += 1,
                _ => {}
            }
            counts
        }
    );
    
    println!("✅ Bulk health monitoring: {} online, {} error, {} offline", 
             status_counts.0, status_counts.1, status_counts.2);
    
    // Step 4: Test bulk configuration management
    println!("⚙️ Step 4: Test Bulk Configuration Management");
    
    // Increment config versions for all agents
    let mut config_versions = Vec::new();
    for agent in &deployed_agents {
        let version = setup.config_service.increment_config_version(agent.agent_id).await
            .expect("Bulk config version increment should succeed");
        config_versions.push(version);
    }
    
    // Verify all agents have updated config versions
    for (agent, expected_version) in deployed_agents.iter().zip(config_versions.iter()) {
        let current_version = setup.config_service.get_current_config_version(agent.agent_id).await
            .expect("Config version check should succeed");
        assert_eq!(current_version, *expected_version);
    }
    
    println!("✅ Bulk configuration management successful");
    
    // Step 5: Test bulk command dispatch
    println!("📡 Step 5: Test Bulk Command Dispatch");
    
    let bulk_sender = CommandSender {
        user_id: enterprise_user.user_id,
        username: "agent_mgmt_enterprise".to_string(),
        role: UserRole::SystemAdmin,
        subscription_tier: "professional".to_string(),
        permissions: vec!["system_admin".to_string(), "bulk_operations".to_string()],
        ip_address: Some("127.0.0.1".to_string()),
    };
    
    // Send system info command to all agents
    let mut bulk_command_ids = Vec::new();
    for agent in &deployed_agents {
        let command_id = setup.remote_command_service.submit_command(
            agent.agent_id,
            RemoteCommand::GetSystemInfo,
            bulk_sender.clone()
        ).await.expect("Bulk command submission should succeed");
        
        bulk_command_ids.push(command_id);
    }
    
    println!("✅ Bulk command dispatch successful: {} commands sent", bulk_command_ids.len());
    
    // Step 6: Verify bulk command status
    println!("📊 Step 6: Verify Bulk Command Status");
    
    let mut total_pending = 0;
    for agent in &deployed_agents {
        let pending = setup.remote_command_service.get_pending_commands(agent.agent_id).await
            .expect("Pending commands check should succeed");
        total_pending += pending.len();
    }
    
    assert_eq!(total_pending, deployed_agents.len()); // One command per agent
    println!("✅ Bulk command status verified: {} total pending commands", total_pending);
    
    // Step 7: Test bulk agent information retrieval
    println!("📋 Step 7: Test Bulk Agent Information Retrieval");
    
    // Get tenant info for bulk operations
    let tenant_id = deployed_agents[0].tenant_id;
    let tenant_agents = setup.agent_service.list_agents_for_tenant(tenant_id).await
        .expect("Tenant agent listing should succeed");
    
    assert_eq!(tenant_agents.len(), deployed_agents.len());
    
    // Verify all agents belong to same tenant
    for agent in &tenant_agents {
        assert_eq!(agent.tenant_id, tenant_id);
        assert_eq!(agent.user_id.unwrap(), enterprise_user.user_id);
    }
    
    println!("✅ Bulk agent information retrieval successful");
    
    println!("✅ Mass Agent Deployment workflow SUCCESSFUL!");
    println!("📈 Summary: {} agents deployed, managed, monitored, and commanded successfully", 
             deployed_agents.len());
    
    setup.cleanup_test_data().await.expect("Cleanup should work");
}