use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

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
    pub user_id: Option<Uuid>, // Link to user who owns this device
    pub hardware_fingerprint: String,
    pub device_name: Option<String>, // User-friendly device name
    pub os_info: serde_json::Value,
    pub status: AgentStatus,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub registered_via_key_id: Option<Uuid>, // Track which API key was used
    pub registered_via_token_id: Option<Uuid>, // Track which token was used
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
    pub api_key: String,     // API key from user profile
    pub device_name: String, // User-friendly device name
    pub hardware_fingerprint: String,
    pub os_info: serde_json::Value,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub agent_id: Uuid,
    pub status: AgentStatus,
}

// API Key management models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub key_id: Uuid,
    pub user_id: Uuid,
    pub key_name: String,
    pub key_prefix: String, // First 8 chars for identification
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub key_name: String,
    pub expires_in_days: Option<i32>, // Optional expiration
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub key_id: Uuid,
    pub api_key: String, // Full API key (only returned once)
    pub key_prefix: String,
    pub key_name: String,
    pub expires_at: Option<DateTime<Utc>>,
}

// Registration token models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RegistrationToken {
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub device_name: String,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRegistrationTokenRequest {
    pub device_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRegistrationTokenResponse {
    pub token_id: Uuid,
    pub registration_token: String, // Full token (only returned once)
    pub device_name: String,
    pub expires_at: DateTime<Utc>,
}

// Alternative registration using token instead of API key
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterAgentWithTokenRequest {
    pub registration_token: String,
    pub hardware_fingerprint: String,
    pub os_info: serde_json::Value,
    pub version: String,
}
