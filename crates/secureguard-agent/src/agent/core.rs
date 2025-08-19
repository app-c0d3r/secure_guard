use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::collector::DataCollector;
use super::monitor::SystemMonitor;
use super::scanner::SecurityScanner;
use crate::communication::Client;
use crate::security::AuthManager;
use crate::utils::config::Config;

/// Main agent core that orchestrates all agent functionality
pub struct AgentCore {
    config: Config,
    agent_id: Uuid,
    client: Client,
    auth_manager: AuthManager,
    system_monitor: SystemMonitor,
    security_scanner: SecurityScanner,
    data_collector: DataCollector,
    is_running: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_id: Uuid,
    pub version: String,
    pub status: ServiceStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub system_health: SystemHealth,
    pub uptime: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemHealth {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_status: bool,
    pub services_running: u32,
}

impl AgentCore {
    /// Create a new agent core instance
    pub async fn new(config: Config) -> Result<Self> {
        let agent_id = Self::get_or_create_agent_id(&config).await?;

        info!("Initializing agent with ID: {}", agent_id);

        let client = Client::new(&config).await?;
        let auth_manager = AuthManager::new(&config)?;
        let system_monitor = SystemMonitor::new(&config)?;
        let security_scanner = SecurityScanner::new(&config)?;
        let data_collector = DataCollector::new(&config)?;

        Ok(Self {
            config,
            agent_id,
            client,
            auth_manager,
            system_monitor,
            security_scanner,
            data_collector,
            is_running: false,
        })
    }

    /// Start all agent services
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting agent services...");

        // Authenticate with server
        self.auth_manager.authenticate().await?;
        info!("Authentication successful");

        // Register agent with server
        self.register_agent().await?;
        info!("Agent registration successful");

        // Start monitoring services
        self.system_monitor.start().await?;
        self.security_scanner.start().await?;
        self.data_collector.start().await?;

        self.is_running = true;
        info!("All agent services started successfully");

        Ok(())
    }

    /// Main agent run loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting main agent loop");

        let mut heartbeat_interval = interval(Duration::from_secs(
            self.config.monitoring.heartbeat_interval,
        ));

        let mut data_collection_interval = interval(Duration::from_secs(
            self.config.monitoring.data_collection_interval,
        ));

        while self.is_running {
            tokio::select! {
                // Send heartbeat
                _ = heartbeat_interval.tick() => {
                    if let Err(e) = self.send_heartbeat().await {
                        warn!("Failed to send heartbeat: {}", e);
                    }
                }

                // Collect and send system data
                _ = data_collection_interval.tick() => {
                    if let Err(e) = self.collect_and_send_data().await {
                        warn!("Failed to collect and send data: {}", e);
                    }
                }

                // Handle incoming messages
                message = self.client.receive_message() => {
                    match message {
                        Ok(msg) => {
                            if let Err(e) = self.handle_server_message(msg).await {
                                error!("Failed to handle server message: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive message: {}", e);
                            // Attempt to reconnect
                            self.reconnect().await?;
                        }
                    }
                }

                // Check for security events
                event = self.security_scanner.check_for_events() => {
                    match event {
                        Ok(Some(security_event)) => {
                            if let Err(e) = self.handle_security_event(security_event).await {
                                error!("Failed to handle security event: {}", e);
                            }
                        }
                        Ok(None) => {} // No events
                        Err(e) => {
                            error!("Security scanner error: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Gracefully shutdown the agent
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down agent...");

        self.is_running = false;

        // Stop all services
        self.system_monitor.stop().await?;
        self.security_scanner.stop().await?;
        self.data_collector.stop().await?;

        // Send final status to server
        if let Err(e) = self.send_shutdown_notification().await {
            warn!("Failed to send shutdown notification: {}", e);
        }

        // Close connection
        self.client.close().await?;

        info!("Agent shutdown complete");
        Ok(())
    }

    async fn get_or_create_agent_id(config: &Config) -> Result<Uuid> {
        // Try to load existing agent ID from config or generate new one
        match config.agent.agent_id.as_ref() {
            Some(id_str) => Uuid::parse_str(id_str)
                .map_err(|e| anyhow::anyhow!("Invalid agent ID in config: {}", e)),
            None => {
                let new_id = Uuid::new_v4();
                info!("Generated new agent ID: {}", new_id);
                // TODO: Save to persistent storage
                Ok(new_id)
            }
        }
    }

    async fn register_agent(&mut self) -> Result<()> {
        use crate::communication::messages::{AgentMessage, RegistrationData};

        let registration_data = RegistrationData {
            agent_id: self.agent_id,
            hostname: self.system_monitor.get_hostname()?,
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: vec![
                "monitoring".to_string(),
                "security_scanning".to_string(),
                "real_time_alerts".to_string(),
            ],
        };

        let message = AgentMessage::Registration(registration_data);
        self.client.send_message(&message).await?;

        debug!("Agent registration message sent");
        Ok(())
    }

    async fn send_heartbeat(&mut self) -> Result<()> {
        use crate::communication::messages::{AgentMessage, HeartbeatData};

        let system_health = self.system_monitor.get_current_health().await?;

        let heartbeat = HeartbeatData {
            agent_id: self.agent_id,
            timestamp: Utc::now(),
            status: ServiceStatus::Running,
            system_health,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let message = AgentMessage::Heartbeat(heartbeat);
        self.client.send_message(&message).await?;

        debug!("Heartbeat sent successfully");
        Ok(())
    }

    async fn collect_and_send_data(&mut self) -> Result<()> {
        use crate::communication::messages::{AgentMessage, SystemInfoData};

        let system_info = self.data_collector.collect_system_data().await?;

        let message = AgentMessage::SystemInfo(SystemInfoData {
            agent_id: self.agent_id,
            timestamp: Utc::now(),
            system_info,
        });

        self.client.send_message(&message).await?;
        debug!("System data sent successfully");

        Ok(())
    }

    async fn handle_server_message(
        &mut self,
        message: crate::communication::messages::ServerMessage,
    ) -> Result<()> {
        use crate::communication::messages::ServerMessage;

        debug!("Received server message: {:?}", message);

        match message {
            ServerMessage::Configuration(config_data) => {
                self.handle_configuration_update(config_data).await?;
            }
            ServerMessage::Command(command_data) => {
                self.handle_command(command_data).await?;
            }
            ServerMessage::UpdateAvailable(update_data) => {
                self.handle_update_notification(update_data).await?;
            }
            ServerMessage::PolicyUpdate(policy_data) => {
                self.handle_policy_update(policy_data).await?;
            }
        }

        Ok(())
    }

    async fn handle_configuration_update(
        &mut self,
        _config_data: crate::communication::messages::ConfigurationData,
    ) -> Result<()> {
        info!("Received configuration update");
        // TODO: Implement configuration update logic
        Ok(())
    }

    async fn handle_command(
        &mut self,
        _command_data: crate::communication::messages::CommandData,
    ) -> Result<()> {
        info!("Received command from server");
        // TODO: Implement command handling
        Ok(())
    }

    async fn handle_update_notification(
        &mut self,
        _update_data: crate::communication::messages::UpdateData,
    ) -> Result<()> {
        info!("Update available notification received");
        // TODO: Implement update handling
        Ok(())
    }

    async fn handle_policy_update(
        &mut self,
        _policy_data: crate::communication::messages::PolicyData,
    ) -> Result<()> {
        info!("Policy update received");
        // TODO: Implement policy update handling
        Ok(())
    }

    async fn handle_security_event(
        &mut self,
        _event: crate::security::SecurityEvent,
    ) -> Result<()> {
        info!("Security event detected");
        // TODO: Implement security event handling
        Ok(())
    }

    async fn send_shutdown_notification(&mut self) -> Result<()> {
        use crate::communication::messages::{AgentMessage, HeartbeatData};

        let system_health = self.system_monitor.get_current_health().await?;

        let heartbeat = HeartbeatData {
            agent_id: self.agent_id,
            timestamp: Utc::now(),
            status: ServiceStatus::Stopping,
            system_health,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let message = AgentMessage::Heartbeat(heartbeat);
        self.client.send_message(&message).await?;

        info!("Shutdown notification sent");
        Ok(())
    }

    async fn reconnect(&mut self) -> Result<()> {
        warn!("Attempting to reconnect to server...");

        // Close existing connection
        self.client.close().await?;

        // Create new client connection
        self.client = Client::new(&self.config).await?;

        // Re-authenticate
        self.auth_manager.authenticate().await?;

        info!("Reconnection successful");
        Ok(())
    }
}
