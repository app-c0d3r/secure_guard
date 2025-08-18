use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use secureguard_shared::{Agent, AgentStatus, RegisterAgentRequest, HeartbeatRequest, SecureGuardError, Result};

pub struct AgentService {
    pool: PgPool,
}

impl AgentService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register_agent(&self, tenant_id: Uuid, request: RegisterAgentRequest) -> Result<Agent> {
        if request.hardware_fingerprint.is_empty() || request.version.is_empty() {
            return Err(SecureGuardError::ValidationError(
                "Hardware fingerprint and version are required".to_string()
            ));
        }

        let existing = sqlx::query!(
            "SELECT agent_id FROM agents.endpoints WHERE hardware_fingerprint = $1",
            request.hardware_fingerprint
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            return Err(SecureGuardError::ValidationError(
                "Agent with this hardware fingerprint already exists".to_string()
            ));
        }

        let agent = sqlx::query_as!(
            Agent,
            r#"
            INSERT INTO agents.endpoints (tenant_id, hardware_fingerprint, os_info, version, status, last_heartbeat)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING agent_id, tenant_id, hardware_fingerprint, os_info, status as "status: AgentStatus", last_heartbeat, version, created_at
            "#,
            tenant_id,
            request.hardware_fingerprint,
            request.os_info,
            request.version,
            AgentStatus::Online as AgentStatus,
            Some(Utc::now())
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(agent)
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
            SELECT agent_id, tenant_id, hardware_fingerprint, os_info, status as "status: AgentStatus", last_heartbeat, version, created_at
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

    pub async fn find_by_id(&self, agent_id: Uuid) -> Result<Option<Agent>> {
        let agent = sqlx::query_as!(
            Agent,
            r#"
            SELECT agent_id, tenant_id, hardware_fingerprint, os_info, status as "status: AgentStatus", last_heartbeat, version, created_at
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