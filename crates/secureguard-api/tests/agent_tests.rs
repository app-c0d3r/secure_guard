use secureguard_api::services::{agent_service::AgentService, test_utils::TestDatabase};
use secureguard_shared::{RegisterAgentRequest, HeartbeatRequest, AgentStatus};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_agent_registration() {
    let test_db = TestDatabase::new().await;
    let agent_service = AgentService::new(test_db.pool.clone());
    let tenant_id = Uuid::new_v4();
    
    let register_request = RegisterAgentRequest {
        hardware_fingerprint: "test-fingerprint-12345".to_string(),
        os_info: json!({
            "os": "Windows 11",
            "version": "22H2",
            "architecture": "x64",
            "hostname": "TEST-PC"
        }),
        version: "1.0.0".to_string(),
    };
    
    let agent = agent_service
        .register_agent(tenant_id, register_request)
        .await
        .unwrap();
    
    assert_eq!(agent.tenant_id, tenant_id);
    assert_eq!(agent.hardware_fingerprint, "test-fingerprint-12345");
    assert_eq!(agent.version, "1.0.0");
    assert!(matches!(agent.status, AgentStatus::Online));
    assert!(agent.last_heartbeat.is_some());
    
    // Verify agent count
    assert_eq!(test_db.count_agents().await, 1);
}

#[tokio::test]
async fn test_duplicate_agent_registration() {
    let test_db = TestDatabase::new().await;
    let agent_service = AgentService::new(test_db.pool.clone());
    let tenant_id = Uuid::new_v4();
    
    let register_request = RegisterAgentRequest {
        hardware_fingerprint: "duplicate-fingerprint".to_string(),
        os_info: json!({"os": "Windows 11"}),
        version: "1.0.0".to_string(),
    };
    
    // First registration should succeed
    agent_service
        .register_agent(tenant_id, register_request.clone())
        .await
        .unwrap();
    
    // Second registration with same fingerprint should fail
    let result = agent_service
        .register_agent(tenant_id, register_request)
        .await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_agent_registration_validation() {
    let test_db = TestDatabase::new().await;
    let agent_service = AgentService::new(test_db.pool.clone());
    let tenant_id = Uuid::new_v4();
    
    // Test empty hardware fingerprint
    let invalid_request = RegisterAgentRequest {
        hardware_fingerprint: "".to_string(),
        os_info: json!({"os": "Windows 11"}),
        version: "1.0.0".to_string(),
    };
    
    let result = agent_service
        .register_agent(tenant_id, invalid_request)
        .await;
    assert!(result.is_err());
    
    // Test empty version
    let invalid_request = RegisterAgentRequest {
        hardware_fingerprint: "test-fingerprint".to_string(),
        os_info: json!({"os": "Windows 11"}),
        version: "".to_string(),
    };
    
    let result = agent_service
        .register_agent(tenant_id, invalid_request)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_agent_heartbeat() {
    let test_db = TestDatabase::new().await;
    let agent_service = AgentService::new(test_db.pool.clone());
    let tenant_id = Uuid::new_v4();
    
    // First register an agent
    let register_request = RegisterAgentRequest {
        hardware_fingerprint: "heartbeat-test-fingerprint".to_string(),
        os_info: json!({"os": "Windows 11"}),
        version: "1.0.0".to_string(),
    };
    
    let agent = agent_service
        .register_agent(tenant_id, register_request)
        .await
        .unwrap();
    
    // Update heartbeat
    let heartbeat_request = HeartbeatRequest {
        agent_id: agent.agent_id,
        status: AgentStatus::Online,
    };
    
    let result = agent_service
        .update_heartbeat(heartbeat_request)
        .await;
    assert!(result.is_ok());
    
    // Test heartbeat for non-existent agent
    let invalid_heartbeat = HeartbeatRequest {
        agent_id: Uuid::new_v4(),
        status: AgentStatus::Online,
    };
    
    let result = agent_service
        .update_heartbeat(invalid_heartbeat)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_agents_for_tenant() {
    let test_db = TestDatabase::new().await;
    let agent_service = AgentService::new(test_db.pool.clone());
    let tenant_id = Uuid::new_v4();
    let other_tenant_id = Uuid::new_v4();
    
    // Register agents for first tenant
    for i in 0..3 {
        let register_request = RegisterAgentRequest {
            hardware_fingerprint: format!("fingerprint-{}", i),
            os_info: json!({"os": "Windows 11", "agent_num": i}),
            version: "1.0.0".to_string(),
        };
        
        agent_service
            .register_agent(tenant_id, register_request)
            .await
            .unwrap();
    }
    
    // Register one agent for second tenant
    let register_request = RegisterAgentRequest {
        hardware_fingerprint: "other-tenant-fingerprint".to_string(),
        os_info: json!({"os": "Linux"}),
        version: "1.0.0".to_string(),
    };
    
    agent_service
        .register_agent(other_tenant_id, register_request)
        .await
        .unwrap();
    
    // List agents for first tenant
    let agents = agent_service
        .list_agents_for_tenant(tenant_id)
        .await
        .unwrap();
    
    assert_eq!(agents.len(), 3);
    for agent in &agents {
        assert_eq!(agent.tenant_id, tenant_id);
    }
    
    // List agents for second tenant
    let other_agents = agent_service
        .list_agents_for_tenant(other_tenant_id)
        .await
        .unwrap();
    
    assert_eq!(other_agents.len(), 1);
    assert_eq!(other_agents[0].tenant_id, other_tenant_id);
}

#[tokio::test]
async fn test_find_agent_by_id() {
    let test_db = TestDatabase::new().await;
    let agent_service = AgentService::new(test_db.pool.clone());
    let tenant_id = Uuid::new_v4();
    
    let register_request = RegisterAgentRequest {
        hardware_fingerprint: "find-by-id-fingerprint".to_string(),
        os_info: json!({"os": "Windows 11"}),
        version: "1.0.0".to_string(),
    };
    
    let registered_agent = agent_service
        .register_agent(tenant_id, register_request)
        .await
        .unwrap();
    
    // Find existing agent
    let found_agent = agent_service
        .find_by_id(registered_agent.agent_id)
        .await
        .unwrap();
    
    assert!(found_agent.is_some());
    let found_agent = found_agent.unwrap();
    assert_eq!(found_agent.agent_id, registered_agent.agent_id);
    assert_eq!(found_agent.hardware_fingerprint, registered_agent.hardware_fingerprint);
    
    // Try to find non-existent agent
    let not_found = agent_service
        .find_by_id(Uuid::new_v4())
        .await
        .unwrap();
    
    assert!(not_found.is_none());
}