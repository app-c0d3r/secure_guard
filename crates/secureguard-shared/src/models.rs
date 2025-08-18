use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::Type;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub agent_id: Uuid,
    pub tenant_id: Uuid,
    pub hardware_fingerprint: String,
    pub os_info: serde_json::Value,
    pub status: AgentStatus,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum AgentStatus {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "error")]
    Error,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tenant {
    pub tenant_id: Uuid,
    pub name: String,
    pub plan_tier: String,
    pub created_at: DateTime<Utc>,
}

// Agent-related request models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub agent_name: String,
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: String,
    pub operating_system: String,
    pub hardware_fingerprint: String,
    pub version: Option<String>,
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHeartbeat {
    pub status: crate::AgentStatus,
    pub system_metrics: Option<crate::SystemMetrics>,
    pub pending_commands: Option<Vec<uuid::Uuid>>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterAgentRequest {
    pub hardware_fingerprint: String,
    pub os_info: serde_json::Value,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub agent_id: Uuid,
    pub status: AgentStatus,
}