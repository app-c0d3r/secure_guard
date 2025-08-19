// Mock implementations for customer lifecycle testing
// These extend existing services with methods needed for testing

use secureguard_api::database::Database;
use secureguard_api::services::{
    user_service::UserService,
    agent_service::AgentService,
    api_key_service::ApiKeyService,
    notification_service::NotificationService,
};
use secureguard_shared::{User, Agent, AgentStatus, Result, SecureGuardError};
use uuid::Uuid;
use serde_json;

// Extended UserService with lifecycle methods
pub struct ExtendedUserService {
    pub user_service: UserService,
    pub database: Database,
}

impl ExtendedUserService {
    pub fn new(user_service: UserService, database: Database) -> Self {
        Self { user_service, database }
    }
    
    pub async fn deactivate_user(&self, user_id: Uuid) -> Result<User> {
        // Update user to inactive status
        let result = sqlx::query!(
            "UPDATE users.users SET is_active = FALSE, updated_at = now() WHERE user_id = $1 RETURNING *",
            user_id
        )
        .fetch_one(self.database.pool())
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        Ok(User {
            user_id: result.user_id,
            username: result.username,
            email: result.email,
            created_at: result.created_at,
            updated_at: result.updated_at,
            is_active: result.is_active,
        })
    }
    
    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        // Delete user and all related data
        sqlx::query!("DELETE FROM users.users WHERE user_id = $1", user_id)
            .execute(self.database.pool())
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    // Delegate other methods to original service
    pub async fn create_user(&self, request: secureguard_shared::CreateUserRequest) -> Result<User> {
        self.user_service.create_user(request).await
    }
    
    pub async fn verify_credentials(&self, username: &str, password: &str) -> Result<Option<User>> {
        self.user_service.verify_credentials(username, password).await
    }
}

// Extended AgentService with lifecycle methods
pub struct ExtendedAgentService {
    pub agent_service: AgentService,
    pub database: Database,
}

impl ExtendedAgentService {
    pub fn new(agent_service: AgentService, database: Database) -> Self {
        Self { agent_service, database }
    }
    
    pub async fn get_user_agents(&self, user_id: Uuid) -> Result<Vec<Agent>> {
        let agents = sqlx::query!(
            r#"
            SELECT 
                a.agent_id, a.tenant_id, ak.user_id,
                a.hardware_fingerprint, null as device_name, a.os_info,
                a.status, a.last_heartbeat, a.version, a.created_at,
                null as registered_via_key_id, null as registered_via_token_id
            FROM agents.endpoints a
            LEFT JOIN users.api_keys ak ON a.registered_via_key_id = ak.key_id
            WHERE ak.user_id = $1
            "#,
            user_id
        )
        .fetch_all(self.database.pool())
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        let mut result = Vec::new();
        for agent in agents {
            let status = match agent.status.as_str() {
                "online" => AgentStatus::Online,
                "offline" => AgentStatus::Offline,
                "error" => AgentStatus::Error,
                _ => AgentStatus::Unknown,
            };
            
            result.push(Agent {
                agent_id: agent.agent_id,
                tenant_id: agent.tenant_id,
                user_id: agent.user_id,
                hardware_fingerprint: agent.hardware_fingerprint,
                device_name: agent.device_name,
                os_info: agent.os_info,
                status,
                last_heartbeat: agent.last_heartbeat,
                version: agent.version,
                created_at: agent.created_at,
                registered_via_key_id: agent.registered_via_key_id,
                registered_via_token_id: agent.registered_via_token_id,
            });
        }
        
        Ok(result)
    }
    
    pub async fn get_all_active_agents(&self) -> Result<Vec<Agent>> {
        let agents = sqlx::query!(
            r#"
            SELECT 
                a.agent_id, a.tenant_id, ak.user_id,
                a.hardware_fingerprint, null as device_name, a.os_info,
                a.status, a.last_heartbeat, a.version, a.created_at,
                null as registered_via_key_id, null as registered_via_token_id
            FROM agents.endpoints a
            LEFT JOIN users.api_keys ak ON a.registered_via_key_id = ak.key_id
            WHERE a.status != 'unknown'
            ORDER BY a.last_heartbeat DESC
            "#
        )
        .fetch_all(self.database.pool())
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        let mut result = Vec::new();
        for agent in agents {
            let status = match agent.status.as_str() {
                "online" => AgentStatus::Online,
                "offline" => AgentStatus::Offline,
                "error" => AgentStatus::Error,
                _ => AgentStatus::Unknown,
            };
            
            result.push(Agent {
                agent_id: agent.agent_id,
                tenant_id: agent.tenant_id,
                user_id: agent.user_id,
                hardware_fingerprint: agent.hardware_fingerprint,
                device_name: agent.device_name,
                os_info: agent.os_info,
                status,
                last_heartbeat: agent.last_heartbeat,
                version: agent.version,
                created_at: agent.created_at,
                registered_via_key_id: agent.registered_via_key_id,
                registered_via_token_id: agent.registered_via_token_id,
            });
        }
        
        Ok(result)
    }
    
    pub async fn update_agent_heartbeat(&self, agent_id: Uuid, status: AgentStatus) -> Result<Agent> {
        let status_str = match status {
            AgentStatus::Online => "online",
            AgentStatus::Offline => "offline",
            AgentStatus::Error => "error",
            AgentStatus::Unknown => "unknown",
        };
        
        let result = sqlx::query!(
            r#"
            UPDATE agents.endpoints 
            SET status = $1, last_heartbeat = now() 
            WHERE agent_id = $2 
            RETURNING 
                agent_id, tenant_id, hardware_fingerprint, os_info,
                status, last_heartbeat, version, created_at
            "#,
            status_str,
            agent_id
        )
        .fetch_one(self.database.pool())
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        let updated_status = match result.status.as_str() {
            "online" => AgentStatus::Online,
            "offline" => AgentStatus::Offline,
            "error" => AgentStatus::Error,
            _ => AgentStatus::Unknown,
        };
        
        Ok(Agent {
            agent_id: result.agent_id,
            tenant_id: result.tenant_id,
            user_id: None, // We'd need a join to get this
            hardware_fingerprint: result.hardware_fingerprint,
            device_name: None,
            os_info: result.os_info,
            status: updated_status,
            last_heartbeat: result.last_heartbeat,
            version: result.version,
            created_at: result.created_at,
            registered_via_key_id: None,
            registered_via_token_id: None,
        })
    }
    
    pub async fn remove_agent(&self, agent_id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM agents.endpoints WHERE agent_id = $1", agent_id)
            .execute(self.database.pool())
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn get_agent_health_status(&self, agent_id: Uuid) -> Result<AgentHealthStatus> {
        let result = sqlx::query!(
            "SELECT status, last_heartbeat FROM agents.endpoints WHERE agent_id = $1",
            agent_id
        )
        .fetch_one(self.database.pool())
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        Ok(AgentHealthStatus {
            agent_id,
            is_online: result.status == "online",
            last_heartbeat: result.last_heartbeat,
        })
    }
    
    pub async fn get_agent_details(&self, agent_id: Uuid) -> Result<Agent> {
        let result = sqlx::query!(
            r#"
            SELECT 
                agent_id, tenant_id, hardware_fingerprint, os_info,
                status, last_heartbeat, version, created_at
            FROM agents.endpoints 
            WHERE agent_id = $1
            "#,
            agent_id
        )
        .fetch_one(self.database.pool())
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        let status = match result.status.as_str() {
            "online" => AgentStatus::Online,
            "offline" => AgentStatus::Offline,
            "error" => AgentStatus::Error,
            _ => AgentStatus::Unknown,
        };
        
        Ok(Agent {
            agent_id: result.agent_id,
            tenant_id: result.tenant_id,
            user_id: None,
            hardware_fingerprint: result.hardware_fingerprint,
            device_name: Some("Monitor-Test-Desktop".to_string()), // Mock for testing
            os_info: result.os_info,
            status,
            last_heartbeat: result.last_heartbeat,
            version: result.version,
            created_at: result.created_at,
            registered_via_key_id: None,
            registered_via_token_id: None,
        })
    }
    
    // Delegate to original service
    pub async fn register_agent_with_api_key(&self, request: secureguard_shared::RegisterAgentRequest) -> Result<Agent> {
        self.agent_service.register_agent_with_api_key(request).await
    }
}

// Extended ApiKeyService with lifecycle methods
pub struct ExtendedApiKeyService {
    pub api_key_service: ApiKeyService,
    pub database: Database,
}

impl ExtendedApiKeyService {
    pub fn new(api_key_service: ApiKeyService, database: Database) -> Self {
        Self { api_key_service, database }
    }
    
    pub async fn deactivate_api_key(&self, user_id: Uuid, key_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE users.api_keys SET is_active = FALSE WHERE user_id = $1 AND key_id = $2",
            user_id, key_id
        )
        .execute(self.database.pool())
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn get_user_api_keys(&self, user_id: Uuid) -> Result<Vec<secureguard_shared::ApiKey>> {
        let keys = sqlx::query!(
            r#"
            SELECT 
                key_id, user_id, key_name, key_prefix, is_active,
                expires_at, last_used, usage_count, created_at
            FROM users.api_keys 
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(self.database.pool())
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        
        let mut result = Vec::new();
        for key in keys {
            result.push(secureguard_shared::ApiKey {
                key_id: key.key_id,
                user_id: key.user_id,
                key_name: key.key_name,
                key_prefix: key.key_prefix,
                is_active: key.is_active,
                expires_at: key.expires_at,
                last_used: key.last_used,
                usage_count: key.usage_count,
                created_at: key.created_at,
            });
        }
        
        Ok(result)
    }
    
    // Delegate to original service
    pub async fn create_api_key(&self, user_id: Uuid, request: secureguard_shared::CreateApiKeyRequest) -> Result<secureguard_shared::CreateApiKeyResponse> {
        self.api_key_service.create_api_key(user_id, request).await
    }
}

// Extended NotificationService with lifecycle methods
pub struct ExtendedNotificationService {
    pub notification_service: NotificationService,
    pub database: Database,
}

impl ExtendedNotificationService {
    pub fn new(notification_service: NotificationService, database: Database) -> Self {
        Self { notification_service, database }
    }
    
    pub async fn send_account_cancellation_notification(
        &self,
        user_id: Uuid,
        email: &str,
        reason: &str
    ) -> Result<()> {
        // Mock implementation for testing
        println!("📧 MOCK EMAIL NOTIFICATION:");
        println!("   To: {}", email);
        println!("   Subject: Account Cancellation Confirmation");
        println!("   Reason: {}", reason);
        println!("   User ID: {}", user_id);
        println!("   Status: Successfully sent (mock)");
        
        // In a real implementation, this would integrate with email service
        // For testing, we just log the notification
        Ok(())
    }
}

// Helper struct for agent health monitoring
#[derive(Debug)]
pub struct AgentHealthStatus {
    pub agent_id: Uuid,
    pub is_online: bool,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
}