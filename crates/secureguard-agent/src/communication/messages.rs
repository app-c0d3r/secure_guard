use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::agent::core::{ServiceStatus, SystemHealth};

/// Messages sent from agent to server
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentMessage {
    Registration(RegistrationData),
    Heartbeat(HeartbeatData),
    SystemInfo(SystemInfoData),
    SecurityEvent(SecurityEventData),
    ThreatAlert(ThreatAlertData),
    LogData(LogData),
}

/// Messages sent from server to agent
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    Configuration(ConfigurationData),
    Command(CommandData),
    UpdateAvailable(UpdateData),
    PolicyUpdate(PolicyData),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationData {
    pub agent_id: Uuid,
    pub hostname: String,
    pub platform: String,
    pub architecture: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatData {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub status: ServiceStatus,
    pub system_health: SystemHealth,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfoData {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub system_info: SystemInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub disk_info: Vec<DiskInfo>,
    pub network_info: NetworkInfo,
    pub process_count: u32,
    pub uptime: u64, // seconds
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuInfo {
    pub usage_percent: f64,
    pub core_count: u32,
    pub frequency: u64,           // MHz
    pub temperature: Option<f64>, // Celsius
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,     // bytes
    pub used: u64,      // bytes
    pub available: u64, // bytes
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,     // bytes
    pub used_space: u64,      // bytes
    pub available_space: u64, // bytes
    pub usage_percent: f64,
    pub filesystem: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub is_up: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityEventData {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub severity: AlertSeverity,
    pub description: String,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SecurityEventType {
    MalwareDetected,
    SuspiciousProcess,
    UnauthorizedAccess,
    FileModification,
    NetworkAnomaly,
    PolicyViolation,
    SystemCompromise,
    LoginAttempt,
    PrivilegeEscalation,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatAlertData {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub threat_type: String,
    pub severity: AlertSeverity,
    pub confidence: f64, // 0.0 to 1.0
    pub affected_resources: Vec<String>,
    pub mitigation_steps: Vec<String>,
    pub raw_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogData {
    pub agent_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub category: String,
    pub message: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

// Server -> Agent messages

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationData {
    pub configuration: HashMap<String, serde_json::Value>,
    pub version: String,
    pub applied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandData {
    pub command_id: Uuid,
    pub command_type: CommandType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout: Option<u64>, // seconds
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandType {
    UpdateConfiguration,
    RunScan,
    CollectLogs,
    RestartService,
    UpdateAgent,
    ExecuteScript,
    GetSystemInfo,
    IsolateSystem,
    QuarantineFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateData {
    pub version: String,
    pub download_url: String,
    pub checksum: String,
    pub checksum_algorithm: String, // SHA256, etc.
    pub release_notes: String,
    pub mandatory: bool,
    pub deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyData {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub policy_version: String,
    pub rules: Vec<PolicyRule>,
    pub effective_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: PolicyRuleType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PolicyRuleType {
    FileIntegrityMonitoring,
    ProcessMonitoring,
    NetworkMonitoring,
    RegistryMonitoring,
    ComplianceCheck,
    ThreatDetection,
}

// Response messages

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    pub command_id: Uuid,
    pub status: CommandStatus,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub execution_time: u64, // milliseconds
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandStatus {
    Success,
    Failed,
    Timeout,
    Unauthorized,
}

impl AgentMessage {
    pub fn get_message_type(&self) -> &'static str {
        match self {
            AgentMessage::Registration(_) => "registration",
            AgentMessage::Heartbeat(_) => "heartbeat",
            AgentMessage::SystemInfo(_) => "system_info",
            AgentMessage::SecurityEvent(_) => "security_event",
            AgentMessage::ThreatAlert(_) => "threat_alert",
            AgentMessage::LogData(_) => "log_data",
        }
    }
}

impl ServerMessage {
    pub fn get_message_type(&self) -> &'static str {
        match self {
            ServerMessage::Configuration(_) => "configuration",
            ServerMessage::Command(_) => "command",
            ServerMessage::UpdateAvailable(_) => "update_available",
            ServerMessage::PolicyUpdate(_) => "policy_update",
        }
    }
}
