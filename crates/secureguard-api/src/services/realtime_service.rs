use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::services::threat_service::ThreatService;
use crate::websocket::connection_manager::ConnectionManager;
use crate::websocket::message_router::MessageRouter;
use secureguard_shared::{
    AgentCommand, AgentMessage, AlertStatus, CreateSecurityEventRequest, DashboardMessage, Result,
    SecureGuardError, SecurityEvent, Severity, SystemMetrics, ThreatAlert,
};

#[derive(Clone)]
pub struct RealtimeService {
    pool: PgPool,
    message_router: MessageRouter,
    threat_service: Arc<RwLock<ThreatService>>,
    active_alerts: Arc<RwLock<Vec<ThreatAlert>>>,
}

impl RealtimeService {
    pub fn new(pool: PgPool, connection_manager: ConnectionManager) -> Self {
        let message_router = MessageRouter::new(connection_manager);
        let threat_service = Arc::new(RwLock::new(ThreatService::with_message_router(
            pool.clone(),
            message_router.clone(),
        )));

        Self {
            pool,
            message_router: message_router.clone(),
            threat_service,
            active_alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Set up default message routing subscribers
        self.message_router.setup_default_subscribers().await;

        // Add custom threat intelligence subscribers
        self.setup_threat_intelligence_subscribers().await;

        info!("Realtime service initialized successfully");
        Ok(())
    }

    // Real-time Security Event Processing
    pub async fn process_security_event(
        &self,
        agent_id: Uuid,
        request: CreateSecurityEventRequest,
    ) -> Result<SecurityEvent> {
        let threat_service = self.threat_service.read().await;
        let event = threat_service
            .create_security_event(agent_id, request)
            .await?;

        // Event will be automatically broadcasted through the ThreatService's message router
        info!(
            "Processed and broadcasted security event {} from agent {}",
            event.event_id, agent_id
        );
        Ok(event)
    }

    // Bulk Event Processing with Real-time Updates
    pub async fn process_bulk_events(
        &self,
        events: Vec<(Uuid, CreateSecurityEventRequest)>,
    ) -> Result<Vec<SecurityEvent>> {
        let mut processed_events = Vec::new();
        let mut high_priority_alerts = Vec::new();

        for (agent_id, event_request) in events {
            match self.process_security_event(agent_id, event_request).await {
                Ok(event) => {
                    // Check if this is a high-priority event that needs immediate attention
                    if matches!(event.severity, Severity::High | Severity::Critical) {
                        high_priority_alerts.push((agent_id, event.clone()));
                    }
                    processed_events.push(event);
                }
                Err(e) => {
                    error!("Failed to process bulk event for agent {}: {}", agent_id, e);
                    continue;
                }
            }
        }

        // Broadcast batch completion summary
        if !processed_events.is_empty() {
            self.broadcast_batch_summary(&processed_events, &high_priority_alerts)
                .await?;
        }

        Ok(processed_events)
    }

    // Emergency Broadcast System
    pub async fn broadcast_emergency_alert(
        &self,
        title: &str,
        message: &str,
        severity: Severity,
        affected_agents: Vec<Uuid>,
    ) -> Result<()> {
        // Send emergency broadcast to all dashboards
        self.message_router
            .broadcast_emergency_alert(title, message, severity.clone())
            .await
            .map_err(|e| {
                SecureGuardError::ValidationError(format!("Emergency broadcast failed: {}", e))
            })?;

        // Log critical system alert
        error!(
            "EMERGENCY ALERT: {} - {} (Affected agents: {})",
            title,
            message,
            affected_agents.len()
        );

        // Store in active alerts
        let emergency_alert = ThreatAlert {
            alert_id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            rule_id: None,
            agent_id: affected_agents.first().copied().unwrap_or(Uuid::new_v4()),
            alert_type: "EMERGENCY".to_string(),
            severity,
            title: title.to_string(),
            description: Some(message.to_string()),
            status: AlertStatus::Open,
            assigned_to: None,
            resolved_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut active_alerts = self.active_alerts.write().await;
        active_alerts.push(emergency_alert);

        Ok(())
    }

    // Agent Command Distribution
    pub async fn send_agent_command(&self, agent_id: Uuid, command: &AgentCommand) -> Result<()> {
        self.message_router
            .route_agent_command(agent_id, command)
            .await
            .map_err(|e| {
                SecureGuardError::ValidationError(format!("Command routing failed: {}", e))
            })?;

        info!(
            "Sent command {} to agent {}",
            command.command_type, agent_id
        );
        Ok(())
    }

    // System Metrics Broadcasting
    pub async fn broadcast_system_metrics(
        &self,
        agent_id: Uuid,
        metrics: &SystemMetrics,
    ) -> Result<()> {
        self.message_router
            .route_system_metrics(agent_id, metrics)
            .await
            .map_err(|e| {
                SecureGuardError::ValidationError(format!("Metrics routing failed: {}", e))
            })?;

        Ok(())
    }

    // Agent Status Updates
    pub async fn update_agent_status(
        &self,
        agent_id: Uuid,
        status: secureguard_shared::AgentStatus,
    ) -> Result<()> {
        let last_seen = chrono::Utc::now();

        self.message_router
            .route_agent_status_update(agent_id, status, last_seen)
            .await
            .map_err(|e| {
                SecureGuardError::ValidationError(format!("Status update failed: {}", e))
            })?;

        info!("Updated agent {} status to {:?}", agent_id, status);
        Ok(())
    }

    // Connection Statistics and Health
    pub async fn get_realtime_stats(&self) -> serde_json::Value {
        let (agent_count, dashboard_count, connected_agents) =
            self.message_router.get_connection_stats().await;
        let active_alerts_count = self.active_alerts.read().await.len();

        serde_json::json!({
            "connections": {
                "agents": agent_count,
                "dashboards": dashboard_count,
                "connected_agent_ids": connected_agents
            },
            "alerts": {
                "active": active_alerts_count
            },
            "timestamp": chrono::Utc::now()
        })
    }

    // Threat Intelligence Integration
    async fn setup_threat_intelligence_subscribers(&self) {
        // Subscribe to critical security events for threat intelligence
        self.message_router
            .add_event_subscriber("ThreatIntelligenceProcessor".to_string(), |event| {
                // Process events that might indicate APT or sophisticated attacks
                match event.event_type.as_str() {
                    "lateral_movement" | "privilege_escalation" | "data_exfiltration" => {
                        info!(
                            "Threat intelligence: Processing high-value event {}",
                            event.event_type
                        );
                        true
                    }
                    _ => false,
                }
            })
            .await;

        // Subscribe to multi-agent correlated attacks
        self.message_router
            .add_alert_subscriber("CorrelationAnalyzer".to_string(), |alert| {
                // Analyze alerts that might be part of coordinated attacks
                if matches!(alert.severity, Severity::High | Severity::Critical) {
                    info!(
                        "Correlation analysis: High-severity alert {} detected",
                        alert.alert_type
                    );
                    true
                } else {
                    false
                }
            })
            .await;

        info!("Threat intelligence subscribers configured");
    }

    async fn broadcast_batch_summary(
        &self,
        processed_events: &[SecurityEvent],
        high_priority_alerts: &[(Uuid, SecurityEvent)],
    ) -> Result<()> {
        let summary = serde_json::json!({
            "batch_processed": {
                "total_events": processed_events.len(),
                "high_priority_events": high_priority_alerts.len(),
                "critical_events": processed_events.iter()
                    .filter(|e| matches!(e.severity, Severity::Critical))
                    .count(),
                "affected_agents": processed_events.iter()
                    .map(|e| e.agent_id)
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                "timestamp": chrono::Utc::now()
            }
        });

        let dashboard_message = DashboardMessage::BatchProcessingSummary { summary };

        // Broadcast the batch summary to all dashboards
        // (We could implement this by extending connection_manager, for now we'll skip the direct broadcast)
        if high_priority_alerts.len() > 5 {
            self.message_router
                .broadcast_emergency_alert(
                    &format!("High Priority Alert Cluster"),
                    &format!(
                        "Processed {} events with {} high-priority alerts across {} agents",
                        processed_events.len(),
                        high_priority_alerts.len(),
                        processed_events
                            .iter()
                            .map(|e| e.agent_id)
                            .collect::<std::collections::HashSet<_>>()
                            .len()
                    ),
                    Severity::High,
                )
                .await
                .map_err(|e| SecureGuardError::ValidationError(e))?;
        }

        Ok(())
    }

    // Cleanup and Maintenance
    pub async fn cleanup_old_alerts(&self, max_age_hours: i64) -> Result<usize> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(max_age_hours);

        let mut active_alerts = self.active_alerts.write().await;
        let initial_count = active_alerts.len();

        active_alerts.retain(|alert| alert.created_at > cutoff_time);
        let cleaned_count = initial_count - active_alerts.len();

        if cleaned_count > 0 {
            info!(
                "Cleaned up {} old alerts older than {} hours",
                cleaned_count, max_age_hours
            );
        }

        Ok(cleaned_count)
    }

    pub fn get_message_router(&self) -> MessageRouter {
        self.message_router.clone()
    }
}

// Background task for periodic maintenance
impl RealtimeService {
    pub async fn start_maintenance_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                // Cleanup old alerts (older than 24 hours)
                if let Err(e) = self.cleanup_old_alerts(24).await {
                    error!("Failed to cleanup old alerts: {}", e);
                }

                // Log connection statistics
                let stats = self.get_realtime_stats().await;
                info!("Realtime service stats: {}", stats);
            }
        });
    }
}
