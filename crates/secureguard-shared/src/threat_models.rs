use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum Severity {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum AlertStatus {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "investigating")]
    Investigating,
    #[serde(rename = "resolved")]
    Resolved,
    #[serde(rename = "false_positive")]
    FalsePositive,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum CommandStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "sent")]
    Sent,
    #[serde(rename = "executing")]
    Executing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "timeout")]
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SecurityEvent {
    pub event_id: Uuid,
    pub agent_id: Uuid,
    pub event_type: String,
    pub severity: Severity,
    pub title: String,
    pub description: Option<String>,
    pub event_data: serde_json::Value,
    pub raw_data: Option<serde_json::Value>,
    pub source_ip: Option<String>,
    pub process_name: Option<String>,
    pub file_path: Option<String>,
    pub user_name: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DetectionRule {
    pub rule_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub severity: Severity,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub enabled: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ThreatAlert {
    pub alert_id: Uuid,
    pub event_id: Uuid,
    pub rule_id: Option<Uuid>,
    pub agent_id: Uuid,
    pub alert_type: String,
    pub severity: Severity,
    pub title: String,
    pub description: Option<String>,
    pub status: AlertStatus,
    pub assigned_to: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentCommand {
    pub command_id: Uuid,
    pub agent_id: Uuid,
    pub issued_by: Uuid,
    pub command_type: String,
    pub command_data: serde_json::Value,
    pub status: CommandStatus,
    pub result: Option<serde_json::Value>,
    pub issued_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemMetrics {
    pub metric_id: Uuid,
    pub agent_id: Uuid,
    pub metric_type: String,
    pub metric_data: serde_json::Value,
    pub collected_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Request/Response DTOs for API endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSecurityEventRequest {
    pub event_type: String,
    pub severity: Severity,
    pub title: String,
    pub description: Option<String>,
    pub event_data: serde_json::Value,
    pub raw_data: Option<serde_json::Value>,
    pub source_ip: Option<String>,
    pub process_name: Option<String>,
    pub file_path: Option<String>,
    pub user_name: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAlertRequest {
    pub event_id: Uuid,
    pub rule_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: Severity,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAlertRequest {
    pub status: Option<AlertStatus>,
    pub assigned_to: Option<Uuid>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCommandRequest {
    pub command_type: String,
    pub command_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    pub command_id: Uuid,
    pub status: CommandStatus,
    pub result: Option<serde_json::Value>,
}

// WebSocket message types for real-time communication

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentMessage {
    // Agent to Server
    SecurityEvent(CreateSecurityEventRequest),
    SecurityEvents {
        agent_id: Uuid,
        events: Vec<CreateSecurityEventRequest>,
    },
    CommandResponse {
        command_id: Uuid,
        status: CommandStatus,
        result: Option<serde_json::Value>,
    },
    Heartbeat {
        agent_id: Uuid,
        status: crate::AgentStatus,
        metrics: Option<SystemMetrics>,
    },
    SystemMetrics {
        cpu_usage: f64,
        memory_usage: f64,
        disk_usage: f64,
        network_connections: u32,
        running_processes: u32,
    },

    // Server to Agent
    Command {
        command_id: Uuid,
        command_type: String,
        command_data: serde_json::Value,
    },
    ConfigUpdate(serde_json::Value),
    RuleUpdate(Vec<DetectionRule>),

    // Bidirectional Protocol Messages
    RegistrationConfirmed {
        agent_id: Uuid,
        configuration: serde_json::Value,
    },
    HeartbeatAck {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    EventsProcessed {
        processed_count: usize,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DashboardMessage {
    AgentStatusUpdate {
        agent_id: Uuid,
        status: crate::AgentStatus,
        last_seen: DateTime<Utc>,
    },
    NewSecurityEvent {
        event: SecurityEvent,
        agent_name: String,
    },
    NewThreatAlert {
        alert: ThreatAlert,
        agent_name: String,
        event_title: String,
    },
    SystemMetricsUpdate {
        agent_id: Uuid,
        metrics: SystemMetrics,
    },
    CommandStatusUpdate {
        command: AgentCommand,
    },
    BatchProcessingSummary {
        summary: serde_json::Value,
    },
}

// Event type definitions for structured logging

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    ProcessStart {
        pid: u32,
        executable: String,
        command_line: String,
        parent_pid: Option<u32>,
    },
    ProcessEnd {
        pid: u32,
        executable: String,
        exit_code: i32,
    },
    FileAccess {
        path: String,
        operation: FileOperation,
        process: String,
        pid: u32,
    },
    NetworkConnection {
        local_port: u16,
        remote_addr: String,
        remote_port: u16,
        protocol: String,
        process: String,
    },
    RegistryModification {
        key: String,
        value: Option<String>,
        operation: RegistryOperation,
        process: String,
    },
    UserLogin {
        username: String,
        session_type: String,
        success: bool,
        source_ip: Option<String>,
    },
    SystemChange {
        component: String,
        description: String,
        details: serde_json::Value,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileOperation {
    Read,
    Write,
    Create,
    Delete,
    Rename,
    Execute,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RegistryOperation {
    Create,
    Modify,
    Delete,
    Query,
}
