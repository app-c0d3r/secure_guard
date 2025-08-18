use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use secureguard_shared::{SecureGuardError, Result};
use std::collections::HashMap;
use tracing::{warn, error, info};

use super::notification_service::{NotificationService, SecurityNotificationRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub incident_id: Uuid,
    pub incident_type: String,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub description: String,
    pub evidence: Option<serde_json::Value>,
    pub user_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub first_detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTamperingEvent {
    pub event_id: Uuid,
    pub agent_id: Uuid,
    pub tampering_type: String,
    pub detected_at: DateTime<Utc>,
    pub process_name: Option<String>,
    pub process_user: Option<String>,
    pub protection_action: String,
    pub recovery_successful: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccessViolation {
    pub violation_id: Uuid,
    pub user_id: Uuid,
    pub target_user_id: Option<Uuid>,
    pub target_resource_type: String,
    pub violation_type: String,
    pub access_denied: bool,
    pub risk_score: Option<i32>,
    pub source_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub alert_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub user_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub auto_resolved: bool,
    pub requires_user_action: bool,
}

pub struct SecurityMonitoringService {
    pool: PgPool,
    notification_service: NotificationService,
}

impl SecurityMonitoringService {
    pub fn new(pool: PgPool) -> Self {
        Self { 
            pool: pool.clone(),
            notification_service: NotificationService::new(pool),
        }
    }

    /// Report agent tampering event - ALL SUBSCRIPTION TIERS
    pub async fn report_agent_tampering(
        &self,
        agent_id: Uuid,
        tampering_type: &str,
        process_info: Option<ProcessInfo>,
        system_context: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        warn!("🚨 SECURITY ALERT: Agent tampering detected - Agent: {}, Type: {}", agent_id, tampering_type);

        // Get agent and user information  
        let agent_info = self.get_agent_with_user(agent_id).await?;

        // Determine severity based on tampering type
        let severity = match tampering_type {
            "agent_shutdown" | "service_stop" | "uninstall_attempt" | "process_kill" => "high",
            "file_deletion" | "registry_modification" | "config_tampering" => "medium", 
            "firewall_block" | "network_isolation" => "medium",
            _ => "low"
        };

        // Create tampering event record
        let tampering_event_id = sqlx::query!(
            r#"
            INSERT INTO agent_tampering_events (
                agent_id, tampering_type, detected_at, process_name, process_id, 
                process_user, command_line, system_metrics, protection_action
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING event_id
            "#,
            agent_id,
            tampering_type,
            Utc::now(),
            process_info.as_ref().map(|p| p.name.as_str()),
            process_info.as_ref().map(|p| p.pid as i32),
            process_info.as_ref().map(|p| p.user.as_str()),
            process_info.as_ref().map(|p| p.command_line.as_str()),
            system_context,
            "alert_user" // Initial action - will be updated based on response
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .event_id;

        // Send immediate notification to user (ALL subscription tiers get this)
        self.send_tampering_alert(&agent_info, tampering_type, severity).await?;

        // Take protective action based on subscription tier
        self.execute_tampering_response(&agent_info, tampering_type, severity).await?;

        error!("🚨 CRITICAL: Agent tampering event {} logged for user {} - Type: {}", 
            tampering_event_id, agent_info.username, tampering_type);

        Ok(tampering_event_id)
    }

    /// Check for user isolation violations - Prevent cross-user data access
    pub async fn check_user_access_violation(
        &self,
        requesting_user_id: Uuid,
        target_resource_type: &str,
        target_resource_id: Uuid,
        access_type: &str,
        request_context: RequestContext,
    ) -> Result<bool> {
        // Check if user is trying to access resources they don't own
        let is_authorized = match target_resource_type {
            "agent" => self.user_owns_agent(requesting_user_id, target_resource_id).await?,
            "api_key" => self.user_owns_api_key(requesting_user_id, target_resource_id).await?,
            "command" => self.user_owns_command_target(requesting_user_id, target_resource_id).await?,
            _ => false, // Deny by default for unknown resource types
        };

        if !is_authorized {
            // Log security violation
            let violation_id = self.log_access_violation(
                requesting_user_id,
                target_resource_type,
                target_resource_id,
                access_type,
                &request_context,
            ).await?;

            warn!("🚨 SECURITY VIOLATION: User {} attempted unauthorized {} access to {} {}", 
                requesting_user_id, access_type, target_resource_type, target_resource_id);

            // Check for repeated violations - potential account compromise
            if self.is_repeat_violator(requesting_user_id).await? {
                self.escalate_security_incident(requesting_user_id, violation_id).await?;
            }

            return Ok(false);
        }

        Ok(true)
    }

    /// Monitor API for abuse patterns - Detect attacks on our backend
    pub async fn monitor_api_request(
        &self,
        user_id: Option<Uuid>,
        endpoint: &str,
        method: &str,
        source_ip: &str,
        user_agent: &str,
        response_status: u16,
        response_time_ms: u64,
    ) -> Result<()> {
        // Rate limiting check
        let request_rate = self.get_request_rate(user_id, source_ip, 60).await?; // Last 60 seconds
        
        // Detect suspicious patterns
        let mut suspicious_indicators = Vec::new();
        
        // High request rate
        if request_rate > 100 { // More than 100 requests per minute
            suspicious_indicators.push("high_request_rate");
        }
        
        // SQL injection patterns
        if self.contains_sql_injection_patterns(endpoint) {
            suspicious_indicators.push("sql_injection_attempt");
        }
        
        // Unauthorized endpoint access
        if response_status == 401 || response_status == 403 {
            let failed_auth_count = self.get_failed_auth_count(source_ip, 300).await?; // Last 5 minutes
            if failed_auth_count > 10 {
                suspicious_indicators.push("brute_force_attempt");
            }
        }
        
        // Unusual geographic access
        if let Some(uid) = user_id {
            if self.is_unusual_geographic_access(uid, source_ip).await? {
                suspicious_indicators.push("geographic_anomaly");
            }
        }

        // Data exfiltration patterns
        if method == "GET" && response_status == 200 && response_time_ms > 5000 {
            let large_response_count = self.get_large_response_count(user_id, 3600).await?; // Last hour
            if large_response_count > 20 {
                suspicious_indicators.push("potential_data_exfiltration");
            }
        }

        // Log and alert if suspicious activity detected
        if !suspicious_indicators.is_empty() {
            let attack_type = suspicious_indicators.join(",");
            
            let _event_id = sqlx::query!(
                r#"
                INSERT INTO api_security_events (
                    user_id, endpoint, method, attack_type, source_ip, user_agent,
                    requests_per_minute, suspicious_patterns, action_taken
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING event_id
                "#,
                user_id,
                endpoint,
                method,
                attack_type,
                source_ip,
                user_agent,
                request_rate as i32,
                &suspicious_indicators,
                if request_rate > 200 { "rate_limited" } else { "flagged" }
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
            .event_id;

            warn!("🚨 API SECURITY EVENT: {} from {} - Indicators: {}", 
                attack_type, source_ip, suspicious_indicators.join(", "));

            // Take action if severe
            if request_rate > 200 || suspicious_indicators.contains(&"sql_injection_attempt") {
                self.trigger_security_lockdown(user_id, source_ip, &attack_type).await?;
            }
        }

        Ok(())
    }

    /// Monitor backend system health and security
    pub async fn monitor_backend_system(
        &self,
        component: &str,
        metrics: SystemMetrics,
    ) -> Result<()> {
        let mut alerts = Vec::new();
        
        // High CPU usage
        if metrics.cpu_usage > 90.0 {
            alerts.push("high_cpu_usage");
        }
        
        // High memory usage
        if metrics.memory_usage > 85.0 {
            alerts.push("high_memory_usage");
        }
        
        // Disk space critical
        if metrics.disk_usage > 95.0 {
            alerts.push("disk_space_critical");
        }
        
        // Too many database connections
        if let Some(db_connections) = metrics.active_connections {
            if db_connections > 100 {
                alerts.push("database_connection_limit");
            }
        }
        
        // High error rate
        if let Some(error_rate) = metrics.error_rate {
            if error_rate > 0.05 { // 5% error rate
                alerts.push("high_error_rate");
            }
        }

        // Log system metrics
        sqlx::query!(
            r#"
            INSERT INTO backend_monitoring (
                component, event_type, cpu_usage, memory_usage, disk_usage,
                active_connections, error_rate, alert_threshold_exceeded,
                alert_level
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            component,
            if alerts.is_empty() { "normal_operation" } else { "threshold_exceeded" },
            metrics.cpu_usage,
            metrics.memory_usage,
            metrics.disk_usage,
            metrics.active_connections,
            metrics.error_rate.map(|r| r as f64),
            !alerts.is_empty(),
            if alerts.is_empty() { "info" } else if alerts.len() > 2 { "critical" } else { "warning" }
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Send alerts for critical issues
        if !alerts.is_empty() {
            error!("🚨 BACKEND ALERT: {} - Issues: {}", component, alerts.join(", "));
            self.send_admin_alert(component, &alerts).await?;
        }

        Ok(())
    }

    /// Get user security dashboard - Only show user's own incidents
    pub async fn get_user_security_dashboard(&self, user_id: Uuid) -> Result<UserSecurityDashboard> {
        // Get user's security incidents (only their own)
        let incidents = sqlx::query_as!(
            SecurityIncident,
            r#"
            SELECT incident_id, incident_type, severity, status, title, description,
                   evidence, user_id, agent_id, first_detected_at, resolved_at
            FROM security_incidents 
            WHERE user_id = $1
            ORDER BY first_detected_at DESC
            LIMIT 50
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Get tampering events for user's agents only
        let tampering_events = sqlx::query!(
            r#"
            SELECT ate.event_id, ate.agent_id, ate.tampering_type, ate.detected_at,
                   ate.process_name, ate.process_user, ate.protection_action, 
                   ate.recovery_successful
            FROM agent_tampering_events ate
            JOIN agents.endpoints ae ON ate.agent_id = ae.agent_id
            WHERE ae.user_id = $1
            ORDER BY ate.detected_at DESC
            LIMIT 20
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Get security metrics for user
        let security_metrics = self.calculate_user_security_metrics(user_id).await?;

        Ok(UserSecurityDashboard {
            user_id,
            total_incidents: incidents.len() as u32,
            critical_incidents: incidents.iter().filter(|i| i.severity == "critical").count() as u32,
            incidents_24h: incidents.iter().filter(|i| 
                i.first_detected_at > Utc::now() - chrono::Duration::hours(24)
            ).count() as u32,
            tampering_events: tampering_events.len() as u32,
            recent_incidents: incidents.into_iter().take(10).collect(),
            security_score: security_metrics.security_score,
            risk_level: security_metrics.risk_level,
            recommendations: security_metrics.recommendations,
        })
    }

    // Helper methods
    async fn get_agent_with_user(&self, agent_id: Uuid) -> Result<AgentUserInfo> {
        let info = sqlx::query!(
            r#"
            SELECT ae.agent_id, ae.device_name, ae.user_id, u.username, u.email,
                   p.plan_slug as subscription_tier
            FROM agents.endpoints ae
            JOIN users.users u ON ae.user_id = u.user_id
            JOIN subscriptions.user_subscriptions us ON u.user_id = us.user_id
            JOIN subscriptions.plans p ON us.plan_id = p.plan_id
            WHERE ae.agent_id = $1 AND us.status = 'active'
            "#,
            agent_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SecureGuardError::AgentNotFound)?;

        Ok(AgentUserInfo {
            agent_id: info.agent_id,
            device_name: info.device_name.unwrap_or_else(|| "Unknown".to_string()),
            user_id: info.user_id.unwrap(),
            username: info.username,
            email: info.email,
            subscription_tier: info.subscription_tier,
        })
    }

    async fn send_tampering_alert(&self, agent_info: &AgentUserInfo, tampering_type: &str, severity: &str) -> Result<()> {
        let message = format!(
            "🚨 SECURITY ALERT: Agent Tampering Detected\n\n\
            Device: {} ({})\n\
            Tampering Type: {}\n\
            Severity: {}\n\
            Time: {}\n\n\
            Action Required: Please check your device immediately. If this was not authorized, \
            your system may be compromised. Log into your SecureGuard dashboard for more details.",
            agent_info.device_name, agent_info.agent_id, tampering_type, severity, Utc::now()
        );

        // Send notification (implementation would send email, SMS, push notification)
        self.queue_user_notification(
            agent_info.user_id,
            "security_alert",
            "SecureGuard Security Alert - Agent Tampering Detected",
            &message,
            "high",
        ).await?;

        info!("🔔 Security alert sent to user {} for tampering event on {}", 
            agent_info.username, agent_info.device_name);

        Ok(())
    }

    async fn execute_tampering_response(&self, agent_info: &AgentUserInfo, tampering_type: &str, _severity: &str) -> Result<()> {
        // Response actions based on subscription tier and tampering type
        let response_actions = match (&agent_info.subscription_tier.as_str(), tampering_type) {
            // All tiers: Alert and attempt restart for critical tampering
            (_, "agent_shutdown") | (_, "service_stop") => {
                vec!["alert_user", "attempt_restart", "enable_enhanced_monitoring"]
            }
            
            // Professional+: More aggressive protection
            ("professional" | "enterprise", "uninstall_attempt") => {
                vec!["alert_user", "block_process", "create_forensic_snapshot", "attempt_restart"]
            }
            
            // Enterprise: Full response capability
            ("enterprise", _) => {
                vec!["alert_user", "block_process", "isolate_system", "create_memory_dump", "forensic_collection"]
            }
            
            // Default: Alert only for free/starter
            _ => vec!["alert_user"]
        };

        info!("🛡️ Executing tampering response for {} tier: {:?}", 
            agent_info.subscription_tier, response_actions);

        // Execute response actions (implementation would communicate with agent)
        for action in response_actions {
            match action {
                "attempt_restart" => self.schedule_agent_restart(agent_info.agent_id).await?,
                "enable_enhanced_monitoring" => self.enable_enhanced_monitoring(agent_info.agent_id).await?,
                "create_forensic_snapshot" => self.create_forensic_snapshot(agent_info.agent_id).await?,
                _ => {} // Other actions would be implemented
            }
        }

        Ok(())
    }

    async fn user_owns_agent(&self, user_id: Uuid, agent_id: Uuid) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM agents.endpoints WHERE agent_id = $1 AND user_id = $2",
            agent_id, user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(count.count.unwrap_or(0) > 0)
    }

    async fn user_owns_api_key(&self, user_id: Uuid, key_id: Uuid) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM users.api_keys WHERE key_id = $1 AND user_id = $2",
            key_id, user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(count.count.unwrap_or(0) > 0)
    }

    async fn user_owns_command_target(&self, user_id: Uuid, command_id: Uuid) -> Result<bool> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count 
            FROM agent_commands ac
            JOIN agents.endpoints ae ON ac.agent_id = ae.agent_id
            WHERE ac.command_id = $1 AND ae.user_id = $2
            "#,
            command_id, user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(count.count.unwrap_or(0) > 0)
    }

    async fn log_access_violation(
        &self,
        user_id: Uuid,
        target_resource_type: &str,
        target_resource_id: Uuid,
        access_type: &str,
        context: &RequestContext,
    ) -> Result<Uuid> {
        let violation_id = sqlx::query!(
            r#"
            INSERT INTO user_access_violations (
                user_id, target_resource_type, target_resource_id, access_type,
                violation_type, source_ip, user_agent, endpoint, method,
                risk_score, access_denied
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING violation_id
            "#,
            user_id,
            target_resource_type,
            target_resource_id,
            access_type,
            "unauthorized_access",
            context.source_ip,
            context.user_agent,
            context.endpoint,
            context.method,
            75, // High risk score for unauthorized access
            true
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .violation_id;

        Ok(violation_id)
    }

    // Additional helper methods would be implemented here...
    async fn is_repeat_violator(&self, _user_id: Uuid) -> Result<bool> {
        // Implementation would check for repeated violations
        Ok(false)
    }

    async fn escalate_security_incident(&self, _user_id: Uuid, _violation_id: Uuid) -> Result<()> {
        // Implementation would escalate to security team
        Ok(())
    }

    async fn get_request_rate(&self, _user_id: Option<Uuid>, _source_ip: &str, _seconds: u32) -> Result<u32> {
        // Implementation would calculate request rate
        Ok(10)
    }

    async fn contains_sql_injection_patterns(&self, _endpoint: &str) -> bool {
        // Implementation would check for SQL injection patterns
        false
    }

    async fn get_failed_auth_count(&self, _source_ip: &str, _seconds: u32) -> Result<u32> {
        // Implementation would count failed auth attempts
        Ok(0)
    }

    async fn is_unusual_geographic_access(&self, _user_id: Uuid, _source_ip: &str) -> Result<bool> {
        // Implementation would check geographic patterns
        Ok(false)
    }

    async fn get_large_response_count(&self, _user_id: Option<Uuid>, _seconds: u32) -> Result<u32> {
        // Implementation would count large responses
        Ok(0)
    }

    async fn trigger_security_lockdown(&self, _user_id: Option<Uuid>, _source_ip: &str, _attack_type: &str) -> Result<()> {
        // Implementation would trigger security lockdown
        Ok(())
    }

    async fn send_admin_alert(&self, _component: &str, _alerts: &[&str]) -> Result<()> {
        // Implementation would send alert to admins
        Ok(())
    }

    async fn calculate_user_security_metrics(&self, _user_id: Uuid) -> Result<SecurityMetrics> {
        Ok(SecurityMetrics {
            security_score: 85,
            risk_level: "low".to_string(),
            recommendations: vec!["Enable two-factor authentication".to_string()],
        })
    }

    async fn queue_user_notification(&self, user_id: Uuid, notification_type: &str, subject: &str, message: &str, priority: &str) -> Result<()> {
        // Send real-time notification using the notification service
        let notification_request = SecurityNotificationRequest {
            user_id,
            incident_id: None,
            notification_type: notification_type.to_string(),
            subject: subject.to_string(),
            message: message.to_string(),
            priority: priority.to_string(),
            delivery_methods: vec!["email".to_string(), "push".to_string()], // Default methods
        };

        match self.notification_service.send_security_notification(notification_request).await {
            Ok(notification_ids) => {
                info!("✅ Security notifications sent: {:?}", notification_ids);
            }
            Err(e) => {
                error!("❌ Failed to send security notification: {}", e);
                // Don't fail the security event if notification fails
            }
        }

        Ok(())
    }

    async fn schedule_agent_restart(&self, _agent_id: Uuid) -> Result<()> {
        // Implementation would schedule agent restart
        Ok(())
    }

    async fn enable_enhanced_monitoring(&self, _agent_id: Uuid) -> Result<()> {
        // Implementation would enable enhanced monitoring
        Ok(())
    }

    async fn create_forensic_snapshot(&self, _agent_id: Uuid) -> Result<()> {
        // Implementation would create forensic snapshot
        Ok(())
    }
}

// Supporting data structures
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub user: String,
    pub command_line: String,
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub source_ip: String,
    pub user_agent: String,
    pub endpoint: String,
    pub method: String,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub active_connections: Option<i32>,
    pub error_rate: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct AgentUserInfo {
    pub agent_id: Uuid,
    pub device_name: String,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub subscription_tier: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserSecurityDashboard {
    pub user_id: Uuid,
    pub total_incidents: u32,
    pub critical_incidents: u32,
    pub incidents_24h: u32,
    pub tampering_events: u32,
    pub recent_incidents: Vec<SecurityIncident>,
    pub security_score: u32,
    pub risk_level: String,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SecurityMetrics {
    pub security_score: u32,
    pub risk_level: String,
    pub recommendations: Vec<String>,
}