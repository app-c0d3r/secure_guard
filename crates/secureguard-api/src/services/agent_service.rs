use super::api_key_service::ApiKeyService;
use super::subscription_service::SubscriptionService;
use chrono::Utc;
use secureguard_shared::{
    Agent, AgentStatus, HeartbeatRequest, RegisterAgentRequest, RegisterAgentWithTokenRequest,
    Result, SecureGuardError,
};
use sqlx::PgPool;
use tracing;
use uuid::Uuid;

pub struct AgentService {
    pool: PgPool,
    api_key_service: ApiKeyService,
    subscription_service: SubscriptionService,
}

impl AgentService {
    pub fn new(pool: PgPool) -> Self {
        let api_key_service = ApiKeyService::new(pool.clone());
        let subscription_service = SubscriptionService::new(pool.clone());
        Self {
            pool,
            api_key_service,
            subscription_service,
        }
    }

    /// Register agent using API key (new method)
    pub async fn register_agent_with_api_key(
        &self,
        request: RegisterAgentRequest,
    ) -> Result<Agent> {
        // Validate API key and get user info
        let (user_id, key_id) = self
            .api_key_service
            .validate_api_key(&request.api_key)
            .await?;

        tracing::info!(
            target: "secureguard_api",
            event = "agent_registration_started",
            user_id = %user_id,
            key_id = %key_id,
            device_name = %request.device_name,
            hardware_fingerprint = %request.hardware_fingerprint,
            version = %request.version,
            "Agent registration process started"
        );

        // Check subscription limits BEFORE registration
        let device_limit_check = self
            .subscription_service
            .can_register_device(user_id)
            .await?;
        if !device_limit_check.allowed {
            tracing::warn!(
                target: "secureguard_api",
                event = "agent_registration_failed",
                user_id = %user_id,
                key_id = %key_id,
                device_name = %request.device_name,
                reason = "subscription_limit_exceeded",
                message = %device_limit_check.message,
                status = "failed",
                "Agent registration failed - subscription limit exceeded"
            );
            return Err(SecureGuardError::SubscriptionLimitExceeded(
                format!("Device registration failed: {}. Please upgrade your subscription to register more devices.", 
                    device_limit_check.message)
            ));
        }

        // Validate required fields
        if request.hardware_fingerprint.is_empty()
            || request.version.is_empty()
            || request.device_name.is_empty()
        {
            return Err(SecureGuardError::ValidationError(
                "Hardware fingerprint, version, and device name are required".to_string(),
            ));
        }

        // Check for existing agent with same hardware fingerprint
        let existing = sqlx::query!(
            "SELECT agent_id FROM agents.endpoints WHERE hardware_fingerprint = $1",
            request.hardware_fingerprint
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            tracing::warn!(
                target: "secureguard_api",
                event = "agent_registration_failed",
                user_id = %user_id,
                key_id = %key_id,
                device_name = %request.device_name,
                hardware_fingerprint = %request.hardware_fingerprint,
                reason = "duplicate_hardware_fingerprint",
                status = "failed",
                "Agent registration failed - duplicate hardware fingerprint"
            );
            return Err(SecureGuardError::ValidationError(
                "Agent with this hardware fingerprint already exists".to_string(),
            ));
        }

        // Get user's tenant_id (assuming users belong to tenants)
        let user_tenant = sqlx::query!(
            "SELECT tenant_id FROM users.users WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // For now, use a default tenant if user doesn't have one
        let tenant_id = user_tenant
            .and_then(|t| t.tenant_id)
            .unwrap_or_else(|| Uuid::new_v4());

        // Insert new agent
        let agent = sqlx::query_as!(
            Agent,
            r#"
            INSERT INTO agents.endpoints (
                tenant_id, user_id, device_name, hardware_fingerprint, os_info, 
                version, status, last_heartbeat, registered_via_key_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING agent_id, tenant_id, user_id, hardware_fingerprint, device_name, os_info, 
                      status as "status: AgentStatus", last_heartbeat, version, created_at,
                      registered_via_key_id, registered_via_token_id
            "#,
            tenant_id,
            user_id,
            request.device_name,
            request.hardware_fingerprint,
            request.os_info,
            request.version,
            AgentStatus::Online as AgentStatus,
            Some(Utc::now()),
            key_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Increment device count in subscription tracking
        if let Err(e) = self
            .subscription_service
            .increment_device_count(user_id)
            .await
        {
            // Log the error but don't fail registration - device is already created
            tracing::warn!("Failed to update device count for user {}: {}", user_id, e);
        }

        tracing::info!(
            target: "secureguard_api",
            event = "agent_registration_success",
            user_id = %user_id,
            agent_id = %agent.agent_id,
            key_id = %key_id,
            device_name = %agent.device_name.as_deref().unwrap_or("unknown"),
            hardware_fingerprint = %agent.hardware_fingerprint,
            version = %agent.version,
            tenant_id = %agent.tenant_id,
            status = "success",
            "Agent registered successfully"
        );

        Ok(agent)
    }

    /// Register agent using one-time token
    pub async fn register_agent_with_token(
        &self,
        request: RegisterAgentWithTokenRequest,
    ) -> Result<Agent> {
        // Validate and consume token
        let (user_id, device_name) = self
            .api_key_service
            .validate_and_consume_token(&request.registration_token)
            .await?;

        // Check subscription limits BEFORE registration
        let device_limit_check = self
            .subscription_service
            .can_register_device(user_id)
            .await?;
        if !device_limit_check.allowed {
            return Err(SecureGuardError::SubscriptionLimitExceeded(
                format!("Device registration failed: {}. Please upgrade your subscription to register more devices.", 
                    device_limit_check.message)
            ));
        }

        // Validate required fields
        if request.hardware_fingerprint.is_empty() || request.version.is_empty() {
            return Err(SecureGuardError::ValidationError(
                "Hardware fingerprint and version are required".to_string(),
            ));
        }

        // Check for existing agent
        let existing = sqlx::query!(
            "SELECT agent_id FROM agents.endpoints WHERE hardware_fingerprint = $1",
            request.hardware_fingerprint
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            return Err(SecureGuardError::ValidationError(
                "Agent with this hardware fingerprint already exists".to_string(),
            ));
        }

        // Get user's tenant_id
        let user_tenant = sqlx::query!(
            "SELECT tenant_id FROM users.users WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let tenant_id = user_tenant
            .and_then(|t| t.tenant_id)
            .unwrap_or_else(|| Uuid::new_v4());

        // Insert new agent
        let agent = sqlx::query_as!(
            Agent,
            r#"
            INSERT INTO agents.endpoints (
                tenant_id, user_id, device_name, hardware_fingerprint, os_info, 
                version, status, last_heartbeat
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING agent_id, tenant_id, user_id, hardware_fingerprint, device_name, os_info, 
                      status as "status: AgentStatus", last_heartbeat, version, created_at,
                      registered_via_key_id, registered_via_token_id
            "#,
            tenant_id,
            user_id,
            device_name,
            request.hardware_fingerprint,
            request.os_info,
            request.version,
            AgentStatus::Online as AgentStatus,
            Some(Utc::now())
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Increment device count in subscription tracking
        if let Err(e) = self
            .subscription_service
            .increment_device_count(user_id)
            .await
        {
            // Log the error but don't fail registration - device is already created
            tracing::warn!("Failed to update device count for user {}: {}", user_id, e);
        }

        Ok(agent)
    }

    /// Legacy method - kept for backward compatibility
    pub async fn register_agent(
        &self,
        tenant_id: Uuid,
        request: RegisterAgentRequest,
    ) -> Result<Agent> {
        return self.register_agent_with_api_key(request).await;
    }

    pub async fn update_heartbeat(&self, request: HeartbeatRequest) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE agents.endpoints SET status = $1, last_heartbeat = $2 WHERE agent_id = $3",
            request.status as AgentStatus,
            Utc::now(),
            request.agent_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(SecureGuardError::AgentNotFound);
        }

        Ok(())
    }

    pub async fn list_agents_for_tenant(&self, tenant_id: Uuid) -> Result<Vec<Agent>> {
        let agents = sqlx::query_as!(
            Agent,
            r#"
            SELECT agent_id, tenant_id, user_id, hardware_fingerprint, device_name, os_info, 
                   status as "status: AgentStatus", last_heartbeat, version, created_at,
                   registered_via_key_id, registered_via_token_id
            FROM agents.endpoints 
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(agents)
    }

    /// List all agents for a specific user
    pub async fn list_agents_for_user(&self, user_id: Uuid) -> Result<Vec<Agent>> {
        let agents = sqlx::query_as!(
            Agent,
            r#"
            SELECT agent_id, tenant_id, user_id, hardware_fingerprint, device_name, os_info, 
                   status as "status: AgentStatus", last_heartbeat, version, created_at,
                   registered_via_key_id, registered_via_token_id
            FROM agents.endpoints 
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(agents)
    }

    pub async fn find_by_id(&self, agent_id: Uuid) -> Result<Option<Agent>> {
        let agent = sqlx::query_as!(
            Agent,
            r#"
            SELECT agent_id, tenant_id, user_id, hardware_fingerprint, device_name, os_info, 
                   status as "status: AgentStatus", last_heartbeat, version, created_at,
                   registered_via_key_id, registered_via_token_id
            FROM agents.endpoints 
            WHERE agent_id = $1
            "#,
            agent_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(agent)
    }
}
