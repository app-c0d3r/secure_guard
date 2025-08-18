use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn, error};

use crate::websocket::connection_manager::ConnectionManager;
use secureguard_shared::{AgentMessage, DashboardMessage, SecurityEvent, ThreatAlert, AgentCommand};

#[derive(Clone)]
pub struct MessageRouter {
    connection_manager: ConnectionManager,
    event_subscribers: Arc<RwLock<Vec<EventSubscriber>>>,
}

type EventHandler = Box<dyn Fn(&SecurityEvent) -> bool + Send + Sync>;
type AlertHandler = Box<dyn Fn(&ThreatAlert) -> bool + Send + Sync>;

pub struct EventSubscriber {
    pub id: Uuid,
    pub name: String,
    pub event_handler: Option<EventHandler>,
    pub alert_handler: Option<AlertHandler>,
}

impl MessageRouter {
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            event_subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Security Event Routing
    pub async fn route_security_event(
        &self,
        agent_id: Uuid,
        event: &SecurityEvent,
        agent_name: &str,
    ) -> Result<(), String> {
        info!("Routing security event {} from agent {}", event.event_type, agent_id);

        // Send to all dashboard connections
        let dashboard_message = DashboardMessage::NewSecurityEvent {
            event: event.clone(),
            agent_name: agent_name.to_string(),
        };

        self.connection_manager
            .send_to_all_dashboards(&dashboard_message)
            .await?;

        // Process event subscribers
        self.process_event_subscribers(event).await;

        Ok(())
    }

    // Threat Alert Routing
    pub async fn route_threat_alert(
        &self,
        alert: &ThreatAlert,
        agent_name: &str,
        event_title: &str,
    ) -> Result<(), String> {
        info!("Routing threat alert {} for agent {}", alert.alert_type, alert.agent_id);

        // Send to all dashboard connections
        let dashboard_message = DashboardMessage::NewThreatAlert {
            alert: alert.clone(),
            agent_name: agent_name.to_string(),
            event_title: event_title.to_string(),
        };

        self.connection_manager
            .send_to_all_dashboards(&dashboard_message)
            .await?;

        // Process alert subscribers
        self.process_alert_subscribers(alert).await;

        Ok(())
    }

    // Agent Status Updates
    pub async fn route_agent_status_update(
        &self,
        agent_id: Uuid,
        status: secureguard_shared::AgentStatus,
        last_seen: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), String> {
        let dashboard_message = DashboardMessage::AgentStatusUpdate {
            agent_id,
            status,
            last_seen,
        };

        self.connection_manager
            .send_to_all_dashboards(&dashboard_message)
            .await?;

        info!("Routed agent status update for agent {}: {:?}", agent_id, status);
        Ok(())
    }

    // Command Routing
    pub async fn route_agent_command(
        &self,
        agent_id: Uuid,
        command: &AgentCommand,
    ) -> Result<(), String> {
        info!("Routing command {} to agent {}", command.command_type, agent_id);

        let agent_message = AgentMessage::Command {
            command_id: command.command_id,
            command_type: command.command_type.clone(),
            command_data: command.command_data.clone(),
        };

        self.connection_manager
            .send_to_agent(agent_id, &agent_message)
            .await?;

        // Notify dashboards about command being sent
        let dashboard_message = DashboardMessage::CommandStatusUpdate {
            command: command.clone(),
        };

        self.connection_manager
            .send_to_all_dashboards(&dashboard_message)
            .await?;

        Ok(())
    }

    // System Metrics Routing
    pub async fn route_system_metrics(
        &self,
        agent_id: Uuid,
        metrics: &secureguard_shared::SystemMetrics,
    ) -> Result<(), String> {
        let dashboard_message = DashboardMessage::SystemMetricsUpdate {
            agent_id,
            metrics: metrics.clone(),
        };

        self.connection_manager
            .send_to_all_dashboards(&dashboard_message)
            .await?;

        Ok(())
    }

    // Event Subscriber Management
    pub async fn add_event_subscriber<F>(&self, name: String, handler: F) -> Uuid
    where
        F: Fn(&SecurityEvent) -> bool + Send + Sync + 'static,
    {
        let subscriber_id = Uuid::new_v4();
        let subscriber = EventSubscriber {
            id: subscriber_id,
            name,
            event_handler: Some(Box::new(handler)),
            alert_handler: None,
        };

        let mut subscribers = self.event_subscribers.write().await;
        subscribers.push(subscriber);

        info!("Added event subscriber {}", subscriber_id);
        subscriber_id
    }

    pub async fn add_alert_subscriber<F>(&self, name: String, handler: F) -> Uuid
    where
        F: Fn(&ThreatAlert) -> bool + Send + Sync + 'static,
    {
        let subscriber_id = Uuid::new_v4();
        let subscriber = EventSubscriber {
            id: subscriber_id,
            name,
            event_handler: None,
            alert_handler: Some(Box::new(handler)),
        };

        let mut subscribers = self.event_subscribers.write().await;
        subscribers.push(subscriber);

        info!("Added alert subscriber {}", subscriber_id);
        subscriber_id
    }

    pub async fn remove_subscriber(&self, subscriber_id: Uuid) -> bool {
        let mut subscribers = self.event_subscribers.write().await;
        let initial_len = subscribers.len();
        subscribers.retain(|s| s.id != subscriber_id);
        
        let removed = subscribers.len() < initial_len;
        if removed {
            info!("Removed subscriber {}", subscriber_id);
        }
        removed
    }

    // Broadcast Emergency Alert
    pub async fn broadcast_emergency_alert(
        &self,
        title: &str,
        message: &str,
        severity: secureguard_shared::Severity,
    ) -> Result<(), String> {
        let emergency_alert = DashboardMessage::NewThreatAlert {
            alert: ThreatAlert {
                alert_id: Uuid::new_v4(),
                event_id: Uuid::new_v4(),
                rule_id: None,
                agent_id: Uuid::new_v4(),
                alert_type: "EMERGENCY".to_string(),
                severity,
                title: title.to_string(),
                description: Some(message.to_string()),
                status: secureguard_shared::AlertStatus::Open,
                assigned_to: None,
                resolved_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            agent_name: "SYSTEM".to_string(),
            event_title: title.to_string(),
        };

        self.connection_manager
            .send_to_all_dashboards(&emergency_alert)
            .await?;

        error!("Emergency alert broadcast: {}", title);
        Ok(())
    }

    // Connection Statistics
    pub async fn get_connection_stats(&self) -> (usize, usize, Vec<Uuid>) {
        let (agent_count, dashboard_count) = self.connection_manager.get_connection_count().await;
        let connected_agents = self.connection_manager.get_connected_agents().await;
        
        (agent_count, dashboard_count, connected_agents)
    }

    // Private helper methods
    async fn process_event_subscribers(&self, event: &SecurityEvent) {
        let subscribers = self.event_subscribers.read().await;
        
        for subscriber in subscribers.iter() {
            if let Some(handler) = &subscriber.event_handler {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handler(event)
                })) {
                    Ok(true) => {
                        info!("Event subscriber {} processed event {}", subscriber.name, event.event_id);
                    }
                    Ok(false) => {
                        // Handler indicated it doesn't want to process this event
                    }
                    Err(_) => {
                        error!("Event subscriber {} panicked processing event {}", subscriber.name, event.event_id);
                    }
                }
            }
        }
    }

    async fn process_alert_subscribers(&self, alert: &ThreatAlert) {
        let subscribers = self.event_subscribers.read().await;
        
        for subscriber in subscribers.iter() {
            if let Some(handler) = &subscriber.alert_handler {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handler(alert)
                })) {
                    Ok(true) => {
                        info!("Alert subscriber {} processed alert {}", subscriber.name, alert.alert_id);
                    }
                    Ok(false) => {
                        // Handler indicated it doesn't want to process this alert
                    }
                    Err(_) => {
                        error!("Alert subscriber {} panicked processing alert {}", subscriber.name, alert.alert_id);
                    }
                }
            }
        }
    }
}

// Default event subscribers for common use cases
impl MessageRouter {
    pub async fn setup_default_subscribers(&self) {
        // High-severity event logger
        self.add_event_subscriber(
            "HighSeverityLogger".to_string(),
            |event| {
                matches!(event.severity, secureguard_shared::Severity::High | secureguard_shared::Severity::Critical)
            }
        ).await;

        // Critical alert notifier
        self.add_alert_subscriber(
            "CriticalAlertNotifier".to_string(),
            |alert| {
                matches!(alert.severity, secureguard_shared::Severity::Critical)
            }
        ).await;

        info!("Set up default message router subscribers");
    }
}