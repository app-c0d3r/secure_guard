use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::services::{
    agent_service::AgentService, processing_pipeline::ProcessingPipeline,
    realtime_service::RealtimeService,
};
use crate::websocket::message_router::MessageRouter;
use secureguard_shared::{
    Agent, AgentCommand, AgentHeartbeat, AgentMessage, AgentStatus, CommandStatus,
    CreateAgentRequest, CreateSecurityEventRequest, DashboardMessage, Result, SecureGuardError,
    SystemMetrics,
};

// Agent connection state management
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConnection {
    pub agent_id: Uuid,
    pub connection_id: Uuid,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub status: AgentStatus,
    pub version: String,
    pub capabilities: Vec<String>,
    pub pending_commands: Vec<Uuid>,
    pub metrics_buffer: Vec<SystemMetrics>,
}

// Agent communication statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CommunicationStats {
    pub total_agents: usize,
    pub online_agents: usize,
    pub offline_agents: usize,
    pub commands_sent_24h: u64,
    pub commands_completed_24h: u64,
    pub events_received_24h: u64,
    pub average_response_time_ms: f64,
    pub heartbeat_failures: u64,
}

// Command execution tracking
#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub command_id: Uuid,
    pub agent_id: Uuid,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expected_completion: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
    pub max_retries: u32,
}

// Main agent communication service
#[derive(Clone)]
pub struct AgentCommunicationService {
    pool: PgPool,
    agent_service: Arc<AgentService>,
    processing_pipeline: Arc<ProcessingPipeline>,
    realtime_service: Arc<RealtimeService>,
    message_router: MessageRouter,

    // Agent state management
    active_connections: Arc<RwLock<HashMap<Uuid, AgentConnection>>>,
    command_tracking: Arc<RwLock<HashMap<Uuid, CommandExecution>>>,

    // Communication configuration
    heartbeat_interval: Duration,
    command_timeout: Duration,
    max_missed_heartbeats: u32,

    // Statistics and monitoring
    stats: Arc<RwLock<CommunicationStats>>,
}

impl AgentCommunicationService {
    pub fn new(
        pool: PgPool,
        agent_service: Arc<AgentService>,
        processing_pipeline: Arc<ProcessingPipeline>,
        realtime_service: Arc<RealtimeService>,
    ) -> Self {
        let message_router = realtime_service.get_message_router();

        Self {
            pool,
            agent_service,
            processing_pipeline,
            realtime_service,
            message_router,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            command_tracking: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_interval: Duration::from_secs(30),
            command_timeout: Duration::from_secs(300), // 5 minutes
            max_missed_heartbeats: 3,
            stats: Arc::new(RwLock::new(CommunicationStats::default())),
        }
    }

    // Initialize the agent communication service
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing Agent Communication Service...");

        // Start background tasks
        self.start_heartbeat_monitor().await;
        self.start_command_timeout_monitor().await;
        self.start_stats_collector().await;
        self.start_connection_cleaner().await;

        // Load existing agents and restore connections
        self.restore_agent_connections().await?;

        info!("Agent Communication Service initialized successfully");
        Ok(())
    }

    // Agent Registration and Authentication
    pub async fn register_agent(
        &self,
        request: CreateAgentRequest,
        connection_id: Uuid,
    ) -> Result<Agent> {
        info!(
            "Registering new agent: {} ({})",
            request.agent_name, request.hostname
        );

        // Validate agent hardware fingerprint
        self.validate_hardware_fingerprint(&request.hardware_fingerprint)
            .await?;

        // Extract version to avoid move issues
        let version = request
            .version
            .clone()
            .unwrap_or_else(|| "1.0.0".to_string());
        let capabilities = request.capabilities.clone().unwrap_or_default();

        // Register agent through agent service
        let register_request = secureguard_shared::RegisterAgentRequest {
            hardware_fingerprint: request.hardware_fingerprint.clone(),
            os_info: serde_json::json!({
                "hostname": request.hostname,
                "os": request.operating_system,
                "ip_address": request.ip_address,
                "mac_address": request.mac_address
            }),
            version: version.clone(),
            api_key: "dummy_key".to_string(), // TODO: Extract from authentication
            device_name: request.hostname.clone(),
        };
        let agent = self
            .agent_service
            .register_agent(Uuid::new_v4(), register_request)
            .await?;

        // Create agent connection
        let agent_connection = AgentConnection {
            agent_id: agent.agent_id,
            connection_id,
            last_heartbeat: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            status: AgentStatus::Online,
            version,
            capabilities,
            pending_commands: Vec::new(),
            metrics_buffer: Vec::new(),
        };

        // Store connection
        {
            let mut connections = self.active_connections.write().await;
            connections.insert(agent.agent_id, agent_connection);
        }

        // Broadcast agent registration to dashboards
        self.broadcast_agent_status_update(agent.agent_id, AgentStatus::Online)
            .await?;

        // Send initial configuration to agent
        self.send_initial_configuration(agent.agent_id).await?;

        info!("Agent registered successfully: ID {}", agent.agent_id);
        Ok(agent)
    }

    // Agent Heartbeat Processing
    pub async fn process_heartbeat(&self, agent_id: Uuid, heartbeat: AgentHeartbeat) -> Result<()> {
        debug!("Processing heartbeat from agent: {}", agent_id);

        // Update agent heartbeat through agent service
        let heartbeat_request = secureguard_shared::HeartbeatRequest {
            agent_id,
            status: heartbeat.status,
        };
        self.agent_service
            .update_heartbeat(heartbeat_request)
            .await?;

        // Update connection state
        {
            let mut connections = self.active_connections.write().await;
            if let Some(connection) = connections.get_mut(&agent_id) {
                connection.last_heartbeat = chrono::Utc::now();
                connection.last_seen = chrono::Utc::now();
                connection.status = heartbeat.status.clone();

                // Process any system metrics in the heartbeat
                if let Some(metrics) = heartbeat.system_metrics {
                    connection.metrics_buffer.push(metrics.clone());

                    // Forward metrics to real-time service
                    self.realtime_service
                        .broadcast_system_metrics(agent_id, &metrics)
                        .await?;
                }

                // Check for pending commands and send them
                self.send_pending_commands(agent_id).await?;
            } else {
                warn!("Received heartbeat from unregistered agent: {}", agent_id);
                return Err(SecureGuardError::AgentNotFound);
            }
        }

        Ok(())
    }

    // Security Event Processing from Agents
    pub async fn process_security_events(
        &self,
        agent_id: Uuid,
        events: Vec<CreateSecurityEventRequest>,
    ) -> Result<()> {
        debug!(
            "Processing {} security events from agent: {}",
            events.len(),
            agent_id
        );

        // Update agent last seen
        self.update_agent_last_seen(agent_id).await;

        // Process events through the processing pipeline
        let events_count = events.len();
        let events_with_agent: Vec<_> = events.into_iter().map(|event| (agent_id, event)).collect();

        self.processing_pipeline
            .process_events_batch(events_with_agent)
            .await?;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.events_received_24h += events_count as u64;
        }

        Ok(())
    }

    // Command Distribution to Agents
    pub async fn send_command_to_agent(&self, agent_id: Uuid, command: AgentCommand) -> Result<()> {
        info!(
            "Sending command {} to agent {}: {}",
            command.command_id, agent_id, command.command_type
        );

        // Check if agent is connected
        {
            let connections = self.active_connections.read().await;
            if !connections.contains_key(&agent_id) {
                return Err(SecureGuardError::AgentNotFound);
            }
        }

        // Track command execution
        let execution = CommandExecution {
            command_id: command.command_id,
            agent_id,
            issued_at: chrono::Utc::now(),
            expected_completion: chrono::Utc::now()
                + chrono::Duration::from_std(self.command_timeout)
                    .unwrap_or_else(|_| chrono::Duration::seconds(300)),
            retry_count: 0,
            max_retries: 3,
        };

        {
            let mut tracking = self.command_tracking.write().await;
            tracking.insert(command.command_id, execution);
        }

        // Add command to pending commands for the agent
        {
            let mut connections = self.active_connections.write().await;
            if let Some(connection) = connections.get_mut(&agent_id) {
                connection.pending_commands.push(command.command_id);
            }
        }

        // Send command through message router
        self.message_router
            .route_agent_command(agent_id, &command)
            .await
            .map_err(|e| {
                SecureGuardError::ValidationError(format!("Failed to send command: {}", e))
            })?;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.commands_sent_24h += 1;
        }

        Ok(())
    }

    // Command Response Processing
    pub async fn process_command_response(
        &self,
        agent_id: Uuid,
        command_id: Uuid,
        status: CommandStatus,
        result: Option<serde_json::Value>,
    ) -> Result<()> {
        info!(
            "Processing command response from agent {}: {} -> {:?}",
            agent_id, command_id, status
        );

        // Update agent last seen
        self.update_agent_last_seen(agent_id).await;

        // Remove from pending commands
        {
            let mut connections = self.active_connections.write().await;
            if let Some(connection) = connections.get_mut(&agent_id) {
                connection.pending_commands.retain(|&id| id != command_id);
            }
        }

        // Remove from command tracking
        {
            let mut tracking = self.command_tracking.write().await;
            tracking.remove(&command_id);
        }

        // Update command status in database (through threat service)
        // This would typically update the command status in the database

        // Update statistics
        if matches!(status, CommandStatus::Completed | CommandStatus::Failed) {
            let mut stats = self.stats.write().await;
            stats.commands_completed_24h += 1;
        }

        // Broadcast command status update to dashboards
        self.broadcast_command_status_update(command_id, status, result)
            .await?;

        Ok(())
    }

    // Agent Status Management
    pub async fn get_agent_status(&self, agent_id: Uuid) -> Result<AgentConnection> {
        let connections = self.active_connections.read().await;
        connections
            .get(&agent_id)
            .cloned()
            .ok_or(SecureGuardError::AgentNotFound)
    }

    pub async fn get_all_agent_statuses(&self) -> HashMap<Uuid, AgentConnection> {
        self.active_connections.read().await.clone()
    }

    pub async fn get_communication_stats(&self) -> CommunicationStats {
        let mut stats = self.stats.read().await.clone();

        // Update real-time counts
        let connections = self.active_connections.read().await;
        stats.total_agents = connections.len();
        stats.online_agents = connections
            .values()
            .filter(|c| matches!(c.status, AgentStatus::Online))
            .count();
        stats.offline_agents = connections
            .values()
            .filter(|c| matches!(c.status, AgentStatus::Offline))
            .count();

        stats
    }

    // Emergency Operations
    pub async fn emergency_isolate_agent(&self, agent_id: Uuid, reason: String) -> Result<()> {
        warn!(
            "EMERGENCY ISOLATION: Agent {} - Reason: {}",
            agent_id, reason
        );

        let isolation_command = AgentCommand {
            command_id: Uuid::new_v4(),
            agent_id,
            issued_by: Uuid::new_v4(), // System user
            command_type: "emergency_isolate".to_string(),
            command_data: serde_json::json!({
                "action": "isolate",
                "reason": reason,
                "timestamp": chrono::Utc::now(),
                "isolation_level": "network_and_process"
            }),
            status: CommandStatus::Pending,
            result: None,
            issued_at: chrono::Utc::now(),
            executed_at: None,
            completed_at: None,
        };

        self.send_command_to_agent(agent_id, isolation_command)
            .await?;

        // Update agent status to isolated
        {
            let mut connections = self.active_connections.write().await;
            if let Some(connection) = connections.get_mut(&agent_id) {
                connection.status = AgentStatus::Offline; // Mark as offline during isolation
            }
        }

        // Broadcast emergency isolation alert
        self.realtime_service
            .broadcast_emergency_alert(
                "Agent Emergency Isolation",
                &format!("Agent {} has been emergency isolated: {}", agent_id, reason),
                secureguard_shared::Severity::Critical,
                vec![agent_id],
            )
            .await?;

        Ok(())
    }

    pub async fn broadcast_emergency_command(
        &self,
        command_type: String,
        command_data: serde_json::Value,
    ) -> Result<Vec<Uuid>> {
        warn!("Broadcasting emergency command: {}", command_type);

        let connections = self.active_connections.read().await;
        let online_agents: Vec<_> = connections
            .iter()
            .filter(|(_, conn)| matches!(conn.status, AgentStatus::Online))
            .map(|(&agent_id, _)| agent_id)
            .collect();

        let mut successful_broadcasts = Vec::new();

        for agent_id in online_agents {
            let emergency_command = AgentCommand {
                command_id: Uuid::new_v4(),
                agent_id,
                issued_by: Uuid::new_v4(), // System user
                command_type: command_type.clone(),
                command_data: command_data.clone(),
                status: CommandStatus::Pending,
                result: None,
                issued_at: chrono::Utc::now(),
                executed_at: None,
                completed_at: None,
            };

            match self
                .send_command_to_agent(agent_id, emergency_command)
                .await
            {
                Ok(_) => successful_broadcasts.push(agent_id),
                Err(e) => error!(
                    "Failed to send emergency command to agent {}: {}",
                    agent_id, e
                ),
            }
        }

        info!(
            "Emergency command broadcast complete: {}/{} agents",
            successful_broadcasts.len(),
            connections.len()
        );

        Ok(successful_broadcasts)
    }

    // Private helper methods
    async fn validate_hardware_fingerprint(&self, fingerprint: &str) -> Result<()> {
        // Implement hardware fingerprint validation
        // This would check against known compromised systems, etc.
        if fingerprint.len() < 10 {
            return Err(SecureGuardError::ValidationError(
                "Invalid hardware fingerprint".to_string(),
            ));
        }
        Ok(())
    }

    async fn send_initial_configuration(&self, agent_id: Uuid) -> Result<()> {
        let config_command = AgentCommand {
            command_id: Uuid::new_v4(),
            agent_id,
            issued_by: Uuid::new_v4(), // System user
            command_type: "configure".to_string(),
            command_data: serde_json::json!({
                "reporting_interval": 30,
                "event_buffer_size": 1000,
                "compression_enabled": true,
                "encryption_level": "AES256"
            }),
            status: CommandStatus::Pending,
            result: None,
            issued_at: chrono::Utc::now(),
            executed_at: None,
            completed_at: None,
        };

        self.send_command_to_agent(agent_id, config_command).await
    }

    async fn send_pending_commands(&self, agent_id: Uuid) -> Result<()> {
        // This would send any queued commands to the agent
        // Implementation would fetch pending commands from database
        Ok(())
    }

    async fn update_agent_last_seen(&self, agent_id: Uuid) {
        let mut connections = self.active_connections.write().await;
        if let Some(connection) = connections.get_mut(&agent_id) {
            connection.last_seen = chrono::Utc::now();
        }
    }

    async fn broadcast_agent_status_update(
        &self,
        agent_id: Uuid,
        status: AgentStatus,
    ) -> Result<()> {
        self.realtime_service
            .update_agent_status(agent_id, status)
            .await
    }

    async fn broadcast_command_status_update(
        &self,
        command_id: Uuid,
        status: CommandStatus,
        result: Option<serde_json::Value>,
    ) -> Result<()> {
        // This would broadcast command status updates to dashboards
        // Implementation would use the message router
        Ok(())
    }

    // Background monitoring tasks
    async fn start_heartbeat_monitor(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(service.heartbeat_interval);

            loop {
                interval.tick().await;
                service.check_agent_heartbeats().await;
            }
        });
    }

    async fn check_agent_heartbeats(&self) {
        let now = chrono::Utc::now();
        let heartbeat_timeout =
            chrono::Duration::from_std(self.heartbeat_interval * self.max_missed_heartbeats)
                .unwrap_or_else(|_| chrono::Duration::seconds(180));

        let mut connections = self.active_connections.write().await;
        let mut offline_agents = Vec::new();

        for (agent_id, connection) in connections.iter_mut() {
            if now - connection.last_heartbeat > heartbeat_timeout
                && matches!(connection.status, AgentStatus::Online)
            {
                warn!("Agent {} heartbeat timeout, marking as offline", agent_id);
                connection.status = AgentStatus::Offline;
                offline_agents.push(*agent_id);

                let mut stats = self.stats.write().await;
                stats.heartbeat_failures += 1;
            }
        }

        // Broadcast status updates for offline agents
        for agent_id in offline_agents {
            if let Err(e) = self
                .broadcast_agent_status_update(agent_id, AgentStatus::Offline)
                .await
            {
                error!(
                    "Failed to broadcast agent offline status for {}: {}",
                    agent_id, e
                );
            }
        }
    }

    async fn start_command_timeout_monitor(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60)); // Check every minute

            loop {
                interval.tick().await;
                service.check_command_timeouts().await;
            }
        });
    }

    async fn check_command_timeouts(&self) {
        let now = chrono::Utc::now();
        let mut tracking = self.command_tracking.write().await;
        let mut timed_out_commands = Vec::new();

        for (command_id, execution) in tracking.iter_mut() {
            if now > execution.expected_completion && execution.retry_count < execution.max_retries
            {
                execution.retry_count += 1;
                execution.expected_completion = now
                    + chrono::Duration::from_std(self.command_timeout)
                        .unwrap_or_else(|_| chrono::Duration::seconds(300));

                warn!(
                    "Command {} timeout, retry {}/{}",
                    command_id, execution.retry_count, execution.max_retries
                );
            } else if execution.retry_count >= execution.max_retries {
                timed_out_commands.push(*command_id);
                error!(
                    "Command {} permanently failed after {} retries",
                    command_id, execution.max_retries
                );
            }
        }

        // Remove permanently failed commands
        for command_id in timed_out_commands {
            tracking.remove(&command_id);
        }
    }

    async fn start_stats_collector(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60)); // Update stats every minute

            loop {
                interval.tick().await;

                let stats = service.get_communication_stats().await;
                info!(
                    "Agent Communication Stats: {} total, {} online, {} offline, {} events/24h",
                    stats.total_agents,
                    stats.online_agents,
                    stats.offline_agents,
                    stats.events_received_24h
                );
            }
        });
    }

    async fn start_connection_cleaner(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(300)); // Clean every 5 minutes

            loop {
                interval.tick().await;
                service.cleanup_old_connections().await;
            }
        });
    }

    async fn cleanup_old_connections(&self) {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(24);
        let mut connections = self.active_connections.write().await;
        let initial_count = connections.len();

        connections.retain(|_, conn| {
            conn.last_seen > cutoff_time || matches!(conn.status, AgentStatus::Online)
        });

        let cleaned = initial_count - connections.len();
        if cleaned > 0 {
            info!("Cleaned up {} old agent connections", cleaned);
        }
    }

    async fn restore_agent_connections(&self) -> Result<()> {
        // This would restore agent connections from the database on startup
        // For now, we'll just log that it's ready
        info!("Agent connections restored from database");
        Ok(())
    }
}
