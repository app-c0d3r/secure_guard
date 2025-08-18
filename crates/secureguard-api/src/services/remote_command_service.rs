use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use secureguard_shared::{SecureGuardError, Result};
use super::subscription_service::SubscriptionService;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteCommand {
    // System Information Commands (Starter+)
    GetSystemInfo,
    GetProcessList,
    GetServices,
    GetNetworkConnections,
    GetInstalledSoftware,
    GetSystemMetrics,

    // File Operations Commands (Professional+)
    GetFileHash { 
        path: String,
        algorithm: Option<String>, // md5, sha1, sha256
    },
    GetFileContent { 
        path: String, 
        max_size: Option<usize>,
        encoding: Option<String>, // utf8, base64
    },
    ListDirectoryContents { 
        path: String,
        recursive: Option<bool>,
        include_hidden: Option<bool>,
    },
    FindFiles { 
        pattern: String, 
        path: String,
        max_results: Option<u32>,
    },
    GetFileMetadata {
        path: String,
    },

    // Security Operations Commands (Professional+)
    RunQuickScan { 
        path: Option<String>,
    },
    RunFullScan,
    GetSecurityLogs { 
        hours: Option<u32>,
        level: Option<String>, // info, warn, error
    },
    GetThreatDetections {
        hours: Option<u32>,
    },
    QuarantineFile {
        path: String,
        reason: String,
    },

    // Forensic Commands (Enterprise + Analyst Role)
    CollectForensicData { 
        scope: ForensicScope,
        evidence_type: Vec<EvidenceType>,
    },
    CreateMemoryDump,
    GetRegistryKeys {
        hive: String,
        key_path: String,
    },
    GetEventLogs {
        log_name: String,
        hours: Option<u32>,
        event_ids: Option<Vec<u32>>,
    },
    CollectNetworkCapture {
        duration_seconds: u32,
        interface: Option<String>,
    },

    // Agent Management Commands (Admin)
    UpdateConfiguration,
    RestartAgent,
    GetAgentStatus,
    EnableFeature { 
        feature: String 
    },
    DisableFeature { 
        feature: String 
    },
    UpdateAgent {
        version: Option<String>,
        force: Option<bool>,
    },
    GetAgentLogs {
        lines: Option<u32>,
        level: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForensicScope {
    Quick,      // Basic system state
    Standard,   // System + processes + network
    Full,       // Everything including memory
    Custom {
        include_memory: bool,
        include_disk: bool,
        include_network: bool,
        include_registry: bool,
        include_logs: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    ProcessList,
    NetworkConnections,
    LoadedModules,
    OpenFiles,
    RegistryKeys,
    EventLogs,
    MemoryDump,
    DiskImage,
    NetworkCapture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResponse {
    SystemInfo(SystemInfo),
    ProcessList(Vec<ProcessInfo>),
    ServicesList(Vec<ServiceInfo>),
    NetworkConnections(Vec<NetworkConnection>),
    InstalledSoftware(Vec<SoftwareInfo>),
    SystemMetrics(SystemMetrics),
    FileHash { path: String, hash: String, algorithm: String },
    FileContent { path: String, content: String, size: usize, encoding: String },
    DirectoryContents(Vec<DirectoryEntry>),
    FileSearch(Vec<FileSearchResult>),
    FileMetadata(FileMetadata),
    ScanResults(ScanResults),
    SecurityLogs(Vec<SecurityLogEntry>),
    ThreatDetections(Vec<ThreatDetection>),
    ForensicData(ForensicEvidence),
    MemoryDump { path: String, size: u64 },
    RegistryData(Vec<RegistryEntry>),
    EventLogs(Vec<EventLogEntry>),
    NetworkCapture { path: String, packets: u32, size: u64 },
    AgentStatus(AgentStatusInfo),
    AgentLogs(Vec<LogEntry>),
    Success { message: String },
    Error { message: String, code: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSender {
    pub user_id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub subscription_tier: String,
    pub permissions: Vec<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    User,           // Basic device owner
    Admin,          // Device administrator  
    Analyst,        // Security analyst
    SystemAdmin,    // Super admin
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommandExecution {
    pub command_id: Uuid,
    pub agent_id: Uuid,
    pub command: RemoteCommand,
    pub sender: CommandSender,
    pub status: CommandStatus,
    pub submitted_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub response: Option<CommandResponse>,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandStatus {
    Queued,
    Sent,
    InProgress,
    Completed,
    Failed,
    Timeout,
    Cancelled,
}

// Response data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub architecture: String,
    pub cpu: String,
    pub total_memory: u64,
    pub available_memory: u64,
    pub uptime: u64,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command_line: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub start_time: DateTime<Utc>,
    pub user: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub start_type: String,
    pub service_type: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub protocol: String,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub pid: u32,
    pub process_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInfo {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_date: Option<DateTime<Utc>>,
    pub install_location: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: HashMap<String, DiskUsage>,
    pub network_stats: NetworkStats,
    pub uptime: u64,
    pub load_average: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

pub struct RemoteCommandService {
    pool: PgPool,
    subscription_service: SubscriptionService,
}

impl RemoteCommandService {
    pub fn new(pool: PgPool) -> Self {
        let subscription_service = SubscriptionService::new(pool.clone());
        Self {
            pool,
            subscription_service,
        }
    }

    /// Submit a remote command to an agent
    pub async fn submit_command(
        &self,
        agent_id: Uuid,
        command: RemoteCommand,
        sender: CommandSender,
    ) -> Result<Uuid> {
        // Verify agent exists and get subscription
        let agent_info = self.get_agent_with_subscription(agent_id).await?;
        
        // Verify command permissions
        self.verify_command_permissions(&command, &sender, &agent_info.subscription_tier).await?;

        // Create command execution record
        let command_id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO agent_commands (
                command_id, agent_id, command_type, command_data, 
                sender_user_id, sender_username, sender_role, sender_ip,
                status, submitted_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            command_id,
            agent_id,
            self.get_command_type(&command),
            serde_json::to_string(&command).map_err(|e| SecureGuardError::InternalError(e.to_string()))?,
            sender.user_id,
            sender.username,
            serde_json::to_string(&sender.role).map_err(|e| SecureGuardError::InternalError(e.to_string()))?,
            sender.ip_address,
            "queued",
            Utc::now()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Log command submission for audit
        self.audit_log_command_submission(&command_id, &command, &sender).await?;

        tracing::info!(
            "Command {} submitted to agent {} by user {} ({})",
            command_id, agent_id, sender.username, sender.role
        );

        Ok(command_id)
    }

    /// Verify if sender can execute the command on the agent
    async fn verify_command_permissions(
        &self,
        command: &RemoteCommand,
        sender: &CommandSender,
        subscription_tier: &str,
    ) -> Result<()> {
        // Check subscription tier requirements
        let required_tier = self.get_required_subscription_tier(command);
        if !self.subscription_tier_allows(subscription_tier, &required_tier) {
            return Err(SecureGuardError::SubscriptionLimitExceeded(
                format!("Command requires {} subscription or higher", required_tier)
            ));
        }

        // Check role permissions
        let allowed = match command {
            // Basic commands - all roles
            RemoteCommand::GetSystemInfo | RemoteCommand::GetAgentStatus => true,

            // System information - Admin+ roles
            RemoteCommand::GetProcessList 
            | RemoteCommand::GetServices 
            | RemoteCommand::GetNetworkConnections 
            | RemoteCommand::GetInstalledSoftware 
            | RemoteCommand::GetSystemMetrics => {
                matches!(sender.role, UserRole::Admin | UserRole::Analyst | UserRole::SystemAdmin)
            }

            // File operations - Admin+ roles for Professional+
            RemoteCommand::GetFileHash { .. } 
            | RemoteCommand::GetFileContent { .. }
            | RemoteCommand::ListDirectoryContents { .. }
            | RemoteCommand::FindFiles { .. }
            | RemoteCommand::GetFileMetadata { .. } => {
                matches!(sender.role, UserRole::Admin | UserRole::Analyst | UserRole::SystemAdmin)
            }

            // Security operations - Analyst+ roles
            RemoteCommand::RunQuickScan { .. }
            | RemoteCommand::RunFullScan
            | RemoteCommand::GetSecurityLogs { .. }
            | RemoteCommand::GetThreatDetections { .. }
            | RemoteCommand::QuarantineFile { .. } => {
                matches!(sender.role, UserRole::Analyst | UserRole::SystemAdmin)
            }

            // Forensic operations - Analyst+ roles with Enterprise subscription
            RemoteCommand::CollectForensicData { .. }
            | RemoteCommand::CreateMemoryDump
            | RemoteCommand::GetRegistryKeys { .. }
            | RemoteCommand::GetEventLogs { .. }
            | RemoteCommand::CollectNetworkCapture { .. } => {
                matches!(sender.role, UserRole::Analyst | UserRole::SystemAdmin) && subscription_tier == "enterprise"
            }

            // Agent management - SystemAdmin only
            RemoteCommand::UpdateConfiguration
            | RemoteCommand::RestartAgent
            | RemoteCommand::EnableFeature { .. }
            | RemoteCommand::DisableFeature { .. }
            | RemoteCommand::UpdateAgent { .. } => {
                matches!(sender.role, UserRole::SystemAdmin)
            }

            // Logs - Admin+ roles
            RemoteCommand::GetAgentLogs { .. } => {
                matches!(sender.role, UserRole::Admin | UserRole::Analyst | UserRole::SystemAdmin)
            }
        };

        if !allowed {
            return Err(SecureGuardError::AuthorizationFailed);
        }

        Ok(())
    }

    fn get_required_subscription_tier(&self, command: &RemoteCommand) -> String {
        match command {
            RemoteCommand::GetSystemInfo | RemoteCommand::GetAgentStatus => "free".to_string(),
            
            RemoteCommand::GetProcessList 
            | RemoteCommand::GetServices 
            | RemoteCommand::GetNetworkConnections 
            | RemoteCommand::GetInstalledSoftware 
            | RemoteCommand::GetSystemMetrics => "starter".to_string(),
            
            RemoteCommand::GetFileHash { .. } 
            | RemoteCommand::GetFileContent { .. }
            | RemoteCommand::ListDirectoryContents { .. }
            | RemoteCommand::FindFiles { .. }
            | RemoteCommand::GetFileMetadata { .. }
            | RemoteCommand::RunQuickScan { .. }
            | RemoteCommand::RunFullScan
            | RemoteCommand::GetSecurityLogs { .. }
            | RemoteCommand::GetThreatDetections { .. }
            | RemoteCommand::QuarantineFile { .. } => "professional".to_string(),
            
            RemoteCommand::CollectForensicData { .. }
            | RemoteCommand::CreateMemoryDump
            | RemoteCommand::GetRegistryKeys { .. }
            | RemoteCommand::GetEventLogs { .. }
            | RemoteCommand::CollectNetworkCapture { .. } => "enterprise".to_string(),
            
            _ => "professional".to_string(),
        }
    }

    fn subscription_tier_allows(&self, user_tier: &str, required_tier: &str) -> bool {
        let tier_hierarchy = ["free", "starter", "professional", "enterprise"];
        
        let user_index = tier_hierarchy.iter().position(|&t| t == user_tier).unwrap_or(0);
        let required_index = tier_hierarchy.iter().position(|&t| t == required_tier).unwrap_or(0);
        
        user_index >= required_index
    }

    /// Get pending commands for an agent
    pub async fn get_pending_commands(&self, agent_id: Uuid) -> Result<Vec<RemoteCommandExecution>> {
        let commands = sqlx::query!(
            r#"
            SELECT 
                command_id, agent_id, command_type, command_data,
                sender_user_id, sender_username, sender_role, sender_ip,
                status, submitted_at, started_at, completed_at,
                response_data, error_message, execution_time_ms
            FROM agent_commands 
            WHERE agent_id = $1 AND status IN ('queued', 'sent')
            ORDER BY submitted_at ASC
            "#,
            agent_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let mut executions = Vec::new();
        for cmd in commands {
            let command: RemoteCommand = serde_json::from_str(&cmd.command_data)
                .map_err(|e| SecureGuardError::InternalError(e.to_string()))?;

            let role: UserRole = serde_json::from_str(&cmd.sender_role)
                .map_err(|e| SecureGuardError::InternalError(e.to_string()))?;

            let response = if let Some(response_data) = &cmd.response_data {
                Some(serde_json::from_str(response_data)
                    .map_err(|e| SecureGuardError::InternalError(e.to_string()))?)
            } else {
                None
            };

            let status = match cmd.status.as_str() {
                "queued" => CommandStatus::Queued,
                "sent" => CommandStatus::Sent,
                "in_progress" => CommandStatus::InProgress,
                "completed" => CommandStatus::Completed,
                "failed" => CommandStatus::Failed,
                "timeout" => CommandStatus::Timeout,
                "cancelled" => CommandStatus::Cancelled,
                _ => CommandStatus::Queued,
            };

            executions.push(RemoteCommandExecution {
                command_id: cmd.command_id,
                agent_id: cmd.agent_id,
                command,
                sender: CommandSender {
                    user_id: cmd.sender_user_id,
                    username: cmd.sender_username,
                    role,
                    subscription_tier: "professional".to_string(), // TODO: Get from user subscription
                    permissions: vec![], // TODO: Get from user permissions
                    ip_address: cmd.sender_ip,
                },
                status,
                submitted_at: cmd.submitted_at,
                started_at: cmd.started_at,
                completed_at: cmd.completed_at,
                response,
                error_message: cmd.error_message,
                execution_time_ms: cmd.execution_time_ms.map(|ms| ms as u64),
            });
        }

        Ok(executions)
    }

    /// Mark command as sent to agent
    pub async fn mark_command_sent(&self, command_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE agent_commands SET status = 'sent', started_at = $1 WHERE command_id = $2",
            Utc::now(),
            command_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Update command with response from agent
    pub async fn update_command_response(
        &self, 
        command_id: Uuid, 
        response: CommandResponse,
        execution_time_ms: u64
    ) -> Result<()> {
        let response_data = serde_json::to_string(&response)
            .map_err(|e| SecureGuardError::InternalError(e.to_string()))?;

        sqlx::query!(
            r#"
            UPDATE agent_commands 
            SET status = 'completed', 
                completed_at = $1,
                response_data = $2,
                execution_time_ms = $3
            WHERE command_id = $4
            "#,
            Utc::now(),
            response_data,
            execution_time_ms as i64,
            command_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Log command completion for audit
        self.audit_log_command_completion(&command_id, &response).await?;

        Ok(())
    }

    /// Mark command as failed
    pub async fn mark_command_failed(&self, command_id: Uuid, error_message: String) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE agent_commands 
            SET status = 'failed', 
                completed_at = $1,
                error_message = $2
            WHERE command_id = $3
            "#,
            Utc::now(),
            error_message,
            command_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get command execution history for an agent
    pub async fn get_command_history(
        &self, 
        agent_id: Uuid, 
        limit: Option<u32>
    ) -> Result<Vec<RemoteCommandExecution>> {
        let limit = limit.unwrap_or(100);
        
        let commands = sqlx::query!(
            r#"
            SELECT 
                command_id, agent_id, command_type, command_data,
                sender_user_id, sender_username, sender_role, sender_ip,
                status, submitted_at, started_at, completed_at,
                response_data, error_message, execution_time_ms
            FROM agent_commands 
            WHERE agent_id = $1
            ORDER BY submitted_at DESC
            LIMIT $2
            "#,
            agent_id,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Convert to RemoteCommandExecution objects (similar to get_pending_commands)
        // Implementation would be similar to above...

        Ok(vec![]) // Simplified for brevity
    }

    // Helper methods
    async fn get_agent_with_subscription(&self, agent_id: Uuid) -> Result<AgentWithSubscription> {
        let agent = sqlx::query!(
            r#"
            SELECT a.agent_id, a.user_id, a.device_name, p.plan_slug as subscription_tier
            FROM agents.endpoints a
            JOIN subscriptions.user_subscriptions us ON a.user_id = us.user_id
            JOIN subscriptions.plans p ON us.plan_id = p.plan_id
            WHERE a.agent_id = $1 AND us.status = 'active'
            "#,
            agent_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SecureGuardError::AgentNotFound)?;

        Ok(AgentWithSubscription {
            agent_id: agent.agent_id,
            user_id: agent.user_id.unwrap(),
            device_name: agent.device_name,
            subscription_tier: agent.subscription_tier,
        })
    }

    fn get_command_type(&self, command: &RemoteCommand) -> String {
        match command {
            RemoteCommand::GetSystemInfo => "get_system_info",
            RemoteCommand::GetProcessList => "get_process_list",
            RemoteCommand::GetFileHash { .. } => "get_file_hash",
            RemoteCommand::GetFileContent { .. } => "get_file_content",
            RemoteCommand::CollectForensicData { .. } => "collect_forensic_data",
            RemoteCommand::RestartAgent => "restart_agent",
            _ => "unknown",
        }.to_string()
    }

    async fn audit_log_command_submission(&self, command_id: &Uuid, command: &RemoteCommand, sender: &CommandSender) -> Result<()> {
        tracing::info!(
            "AUDIT: Command {} submitted by {} (role: {:?}) - Type: {}",
            command_id, sender.username, sender.role, self.get_command_type(command)
        );
        Ok(())
    }

    async fn audit_log_command_completion(&self, command_id: &Uuid, response: &CommandResponse) -> Result<()> {
        tracing::info!(
            "AUDIT: Command {} completed successfully - Response type: {}",
            command_id, 
            match response {
                CommandResponse::SystemInfo(_) => "SystemInfo",
                CommandResponse::ProcessList(_) => "ProcessList", 
                CommandResponse::Error { .. } => "Error",
                _ => "Other"
            }
        );
        Ok(())
    }
}

#[derive(Debug)]
struct AgentWithSubscription {
    agent_id: Uuid,
    user_id: Uuid,
    device_name: Option<String>,
    subscription_tier: String,
}

// Additional response types would be defined here...
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified: DateTime<Utc>,
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub path: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub accessed: DateTime<Utc>,
    pub permissions: String,
    pub owner: String,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResults {
    pub scan_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub files_scanned: u64,
    pub threats_found: u32,
    pub threats: Vec<ThreatDetection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub source: String,
    pub message: String,
    pub event_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetection {
    pub id: Uuid,
    pub threat_type: String,
    pub severity: String,
    pub file_path: String,
    pub detected_at: DateTime<Utc>,
    pub signature: String,
    pub action_taken: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicEvidence {
    pub evidence_id: Uuid,
    pub collected_at: DateTime<Utc>,
    pub scope: ForensicScope,
    pub evidence_files: Vec<EvidenceFile>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceFile {
    pub file_type: EvidenceType,
    pub file_path: String,
    pub file_size: u64,
    pub hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub key_path: String,
    pub value_name: String,
    pub value_type: String,
    pub value_data: String,
    pub modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub log_name: String,
    pub event_id: u32,
    pub level: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub message: String,
    pub user: Option<String>,
    pub computer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusInfo {
    pub agent_id: Uuid,
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub last_heartbeat: DateTime<Utc>,
    pub active_features: Vec<String>,
    pub system_metrics: SystemMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub module: String,
    pub message: String,
    pub context: Option<HashMap<String, serde_json::Value>>,
}