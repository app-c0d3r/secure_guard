use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use secureguard_shared::{
    SecurityEvent, DetectionRule, ThreatAlert, AgentCommand, SystemMetrics,
    CreateSecurityEventRequest, CreateAlertRequest, UpdateAlertRequest, CreateCommandRequest,
    Severity, AlertStatus, CommandStatus,
    SecureGuardError, Result
};
use crate::websocket::message_router::MessageRouter;

#[derive(Debug)]
struct TimelineRow {
    pub hour: Option<DateTime<Utc>>,
    pub event_type: String,
    pub severity: String,
    pub event_count: Option<i64>,
}

pub struct ThreatService {
    pool: PgPool,
    message_router: Option<MessageRouter>,
}

impl ThreatService {
    pub fn new(pool: PgPool) -> Self {
        Self { 
            pool,
            message_router: None,
        }
    }

    pub fn with_message_router(pool: PgPool, message_router: MessageRouter) -> Self {
        Self { 
            pool,
            message_router: Some(message_router),
        }
    }

    // Security Events
    pub async fn create_security_event(&self, agent_id: Uuid, request: CreateSecurityEventRequest) -> Result<SecurityEvent> {
        let event = sqlx::query_as!(
            SecurityEvent,
            r#"
            INSERT INTO threats.security_events 
            (agent_id, event_type, severity, title, description, event_data, raw_data, 
             source_ip, process_name, file_path, user_name, occurred_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING event_id, agent_id, event_type, severity as "severity: Severity", 
                     title, description, event_data, raw_data, 
                     source_ip, process_name, file_path, user_name, occurred_at, created_at
            "#,
            agent_id,
            request.event_type,
            match request.severity {
                Severity::Low => "low",
                Severity::Medium => "medium",
                Severity::High => "high",
                Severity::Critical => "critical",
            },
            request.title,
            request.description,
            request.event_data,
            request.raw_data,
            request.source_ip,
            request.process_name,
            request.file_path,
            request.user_name,
            request.occurred_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Process detection rules for this event
        self.process_detection_rules(&event).await?;

        // Broadcast event in real-time (if WebSocket router is available)
        if let Some(router) = &self.message_router {
            if let Err(e) = router.route_security_event(agent_id, &event, "Agent").await {
                tracing::warn!("Failed to broadcast security event: {}", e);
            }
        }

        Ok(event)
    }

    pub async fn get_security_events(&self, agent_id: Option<Uuid>, limit: Option<i64>) -> Result<Vec<SecurityEvent>> {
        let limit = limit.unwrap_or(100);
        
        let events = if let Some(agent_id) = agent_id {
            sqlx::query_as!(
                SecurityEvent,
                r#"
                SELECT event_id, agent_id, event_type, severity as "severity: Severity",
                       title, description, event_data, raw_data,
                       source_ip, process_name, file_path, user_name, occurred_at, created_at
                FROM threats.security_events
                WHERE agent_id = $1
                ORDER BY occurred_at DESC
                LIMIT $2
                "#,
                agent_id,
                limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query_as!(
                SecurityEvent,
                r#"
                SELECT event_id, agent_id, event_type, severity as "severity: Severity",
                       title, description, event_data, raw_data,
                       source_ip, process_name, file_path, user_name, occurred_at, created_at
                FROM threats.security_events
                ORDER BY occurred_at DESC
                LIMIT $1
                "#,
                limit
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        };

        Ok(events)
    }

    // Detection Rules
    pub async fn create_detection_rule(&self, rule: DetectionRule, created_by: Uuid) -> Result<DetectionRule> {
        let rule = sqlx::query_as!(
            DetectionRule,
            r#"
            INSERT INTO threats.detection_rules 
            (name, description, rule_type, severity, conditions, actions, enabled, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING rule_id, name, description, rule_type, severity as "severity: Severity",
                     conditions, actions, enabled, created_by, created_at, updated_at
            "#,
            rule.name,
            rule.description,
            rule.rule_type,
            match rule.severity {
                Severity::Low => "low",
                Severity::Medium => "medium", 
                Severity::High => "high",
                Severity::Critical => "critical",
            },
            rule.conditions,
            rule.actions,
            rule.enabled,
            created_by
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(rule)
    }

    pub async fn get_detection_rules(&self, enabled_only: bool) -> Result<Vec<DetectionRule>> {
        let rules = if enabled_only {
            sqlx::query_as!(
                DetectionRule,
                r#"
                SELECT rule_id, name, description, rule_type, severity as "severity: Severity",
                       conditions, actions, enabled, created_by, created_at, updated_at
                FROM threats.detection_rules
                WHERE enabled = true
                ORDER BY created_at DESC
                "#
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query_as!(
                DetectionRule,
                r#"
                SELECT rule_id, name, description, rule_type, severity as "severity: Severity",
                       conditions, actions, enabled, created_by, created_at, updated_at
                FROM threats.detection_rules
                ORDER BY created_at DESC
                "#
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        };

        Ok(rules)
    }

    // Threat Alerts
    pub async fn create_alert(&self, agent_id: Uuid, request: CreateAlertRequest) -> Result<ThreatAlert> {
        let alert = sqlx::query_as!(
            ThreatAlert,
            r#"
            INSERT INTO threats.alerts 
            (event_id, rule_id, agent_id, alert_type, severity, title, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING alert_id, event_id, rule_id, agent_id, alert_type, 
                     severity as "severity: Severity", title, description,
                     status as "status: AlertStatus", assigned_to, resolved_at, created_at, updated_at
            "#,
            request.event_id,
            request.rule_id,
            agent_id,
            request.alert_type,
            match request.severity {
                Severity::Low => "low",
                Severity::Medium => "medium",
                Severity::High => "high", 
                Severity::Critical => "critical",
            },
            request.title,
            request.description
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(alert)
    }

    pub async fn update_alert(&self, alert_id: Uuid, request: UpdateAlertRequest) -> Result<ThreatAlert> {
        let alert = sqlx::query_as!(
            ThreatAlert,
            r#"
            UPDATE threats.alerts 
            SET status = COALESCE($2, status),
                assigned_to = COALESCE($3, assigned_to),
                description = COALESCE($4, description),
                resolved_at = CASE WHEN $2 = 'resolved' THEN now() ELSE resolved_at END,
                updated_at = now()
            WHERE alert_id = $1
            RETURNING alert_id, event_id, rule_id, agent_id, alert_type,
                     severity as "severity: Severity", title, description,
                     status as "status: AlertStatus", assigned_to, resolved_at, created_at, updated_at
            "#,
            alert_id,
            request.status.as_ref().map(|s| match s {
                AlertStatus::Open => "open",
                AlertStatus::Investigating => "investigating", 
                AlertStatus::Resolved => "resolved",
                AlertStatus::FalsePositive => "false_positive",
            }),
            request.assigned_to,
            request.description
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(alert)
    }

    pub async fn get_alerts(&self, agent_id: Option<Uuid>, status: Option<AlertStatus>) -> Result<Vec<ThreatAlert>> {
        let alerts = match (agent_id, status) {
            (Some(agent_id), Some(status)) => {
                sqlx::query_as!(
                    ThreatAlert,
                    r#"
                    SELECT alert_id, event_id, rule_id, agent_id, alert_type,
                           severity as "severity: Severity", title, description,
                           status as "status: AlertStatus", assigned_to, resolved_at, created_at, updated_at
                    FROM threats.alerts
                    WHERE agent_id = $1 AND status = $2
                    ORDER BY created_at DESC
                    "#,
                    agent_id,
                    status as AlertStatus
                )
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
            },
            (Some(agent_id), None) => {
                sqlx::query_as!(
                    ThreatAlert,
                    r#"
                    SELECT alert_id, event_id, rule_id, agent_id, alert_type,
                           severity as "severity: Severity", title, description,
                           status as "status: AlertStatus", assigned_to, resolved_at, created_at, updated_at
                    FROM threats.alerts
                    WHERE agent_id = $1
                    ORDER BY created_at DESC
                    "#,
                    agent_id
                )
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
            },
            (None, Some(status)) => {
                sqlx::query_as!(
                    ThreatAlert,
                    r#"
                    SELECT alert_id, event_id, rule_id, agent_id, alert_type,
                           severity as "severity: Severity", title, description,
                           status as "status: AlertStatus", assigned_to, resolved_at, created_at, updated_at
                    FROM threats.alerts
                    WHERE status = $1
                    ORDER BY created_at DESC
                    "#,
                    status as AlertStatus
                )
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
            },
            (None, None) => {
                sqlx::query_as!(
                    ThreatAlert,
                    r#"
                    SELECT alert_id, event_id, rule_id, agent_id, alert_type,
                           severity as "severity: Severity", title, description,
                           status as "status: AlertStatus", assigned_to, resolved_at, created_at, updated_at
                    FROM threats.alerts
                    ORDER BY created_at DESC
                    LIMIT 100
                    "#
                )
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
            }
        };

        Ok(alerts)
    }

    // Agent Commands
    pub async fn create_command(&self, agent_id: Uuid, issued_by: Uuid, request: CreateCommandRequest) -> Result<AgentCommand> {
        let command = sqlx::query_as!(
            AgentCommand,
            r#"
            INSERT INTO threats.agent_commands 
            (agent_id, issued_by, command_type, command_data)
            VALUES ($1, $2, $3, $4)
            RETURNING command_id, agent_id, issued_by, command_type, command_data,
                     status as "status: CommandStatus", result, issued_at, executed_at, completed_at
            "#,
            agent_id,
            issued_by,
            request.command_type,
            request.command_data
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(command)
    }

    pub async fn update_command_status(&self, command_id: Uuid, status: CommandStatus, result: Option<serde_json::Value>) -> Result<AgentCommand> {
        let status_str = match status {
            CommandStatus::Pending => "pending",
            CommandStatus::Sent => "sent",
            CommandStatus::Executing => "executing",
            CommandStatus::Completed => "completed",
            CommandStatus::Failed => "failed", 
            CommandStatus::Timeout => "timeout",
        };
        
        let command = sqlx::query_as!(
            AgentCommand,
            r#"
            UPDATE threats.agent_commands 
            SET status = $2,
                result = COALESCE($3, result),
                executed_at = CASE WHEN $4 = 'executing' THEN now() ELSE executed_at END,
                completed_at = CASE WHEN $5 IN ('completed', 'failed', 'timeout') THEN now() ELSE completed_at END
            WHERE command_id = $1
            RETURNING command_id, agent_id, issued_by, command_type, command_data,
                     status as "status: CommandStatus", result, issued_at, executed_at, completed_at
            "#,
            command_id,
            status_str,
            result,
            status_str,
            status_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(command)
    }

    // Detection Rule Processing
    async fn process_detection_rules(&self, event: &SecurityEvent) -> Result<()> {
        let rules = self.get_detection_rules(true).await?;
        
        for rule in rules {
            if self.evaluate_rule(&rule, event).await? {
                let alert_request = CreateAlertRequest {
                    event_id: event.event_id,
                    rule_id: Some(rule.rule_id),
                    alert_type: format!("Rule Match: {}", rule.name),
                    severity: rule.severity.clone(),
                    title: format!("Security Alert: {}", rule.name),
                    description: rule.description.clone(),
                };
                
                let alert = self.create_alert(event.agent_id, alert_request).await?;
                
                // Broadcast alert in real-time
                if let Some(router) = &self.message_router {
                    if let Err(e) = router.route_threat_alert(&alert, "Agent", &event.title).await {
                        tracing::warn!("Failed to broadcast threat alert: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn evaluate_rule(&self, rule: &DetectionRule, event: &SecurityEvent) -> Result<bool> {
        // Advanced rule evaluation engine
        
        // Check if event type matches rule type
        if rule.rule_type != "all" && rule.rule_type != event.event_type {
            return Ok(false);
        }
        
        // Parse rule conditions
        let conditions = rule.conditions.as_object()
            .ok_or_else(|| SecureGuardError::ValidationError("Invalid rule conditions format".to_string()))?;
        
        // Evaluate different condition types
        match rule.rule_type.as_str() {
            "process" => self.evaluate_process_conditions(conditions, event).await,
            "file" => self.evaluate_file_conditions(conditions, event).await,
            "network" => self.evaluate_network_conditions(conditions, event).await,
            "registry" => self.evaluate_registry_conditions(conditions, event).await,
            "authentication" => self.evaluate_auth_conditions(conditions, event).await,
            _ => self.evaluate_generic_conditions(conditions, event).await,
        }
    }
    
    async fn evaluate_process_conditions(&self, conditions: &serde_json::Map<String, serde_json::Value>, event: &SecurityEvent) -> Result<bool> {
        // Check suspicious process paths
        if let Some(suspicious_paths) = conditions.get("process_path") {
            if let Some(paths) = suspicious_paths.as_array() {
                if let Some(process_name) = &event.process_name {
                    for path in paths {
                        if let Some(path_str) = path.as_str() {
                            if process_name.to_lowercase().contains(&path_str.to_lowercase()) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        // Check file extensions
        if let Some(extensions) = conditions.get("extensions") {
            if let Some(exts) = extensions.as_array() {
                if let Some(file_path) = &event.file_path {
                    for ext in exts {
                        if let Some(ext_str) = ext.as_str() {
                            if file_path.to_lowercase().ends_with(&ext_str.to_lowercase()) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    async fn evaluate_file_conditions(&self, conditions: &serde_json::Map<String, serde_json::Value>, event: &SecurityEvent) -> Result<bool> {
        // Check critical system file modifications
        if let Some(protected_paths) = conditions.get("file_paths") {
            if let Some(paths) = protected_paths.as_array() {
                if let Some(file_path) = &event.file_path {
                    for path in paths {
                        if let Some(path_str) = path.as_str() {
                            if file_path.to_lowercase().starts_with(&path_str.to_lowercase()) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        // Check file operations
        if let Some(operations) = conditions.get("operations") {
            if let Some(ops) = operations.as_array() {
                if let Some(event_data) = event.event_data.as_object() {
                    if let Some(operation) = event_data.get("operation") {
                        if let Some(op_str) = operation.as_str() {
                            for op in ops {
                                if let Some(expected_op) = op.as_str() {
                                    if op_str.eq_ignore_ascii_case(expected_op) {
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    async fn evaluate_network_conditions(&self, conditions: &serde_json::Map<String, serde_json::Value>, event: &SecurityEvent) -> Result<bool> {
        // Check suspicious IP connections
        if let Some(remote_ips) = conditions.get("remote_ips") {
            if let Some(ips) = remote_ips.as_array() {
                if let Some(source_ip) = &event.source_ip {
                    for ip_category in ips {
                        if let Some(category) = ip_category.as_str() {
                            match category {
                                "tor_exit_nodes" => {
                                    // Simplified TOR detection - in production, use threat intelligence feeds
                                    if self.is_tor_exit_node(source_ip).await? {
                                        return Ok(true);
                                    }
                                },
                                "malware_c2" => {
                                    // Check against known C2 servers
                                    if self.is_known_malware_c2(source_ip).await? {
                                        return Ok(true);
                                    }
                                },
                                _ => continue,
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    async fn evaluate_registry_conditions(&self, conditions: &serde_json::Map<String, serde_json::Value>, event: &SecurityEvent) -> Result<bool> {
        // Check registry persistence locations
        if let Some(registry_keys) = conditions.get("registry_keys") {
            if let Some(keys) = registry_keys.as_array() {
                if let Some(event_data) = event.event_data.as_object() {
                    if let Some(registry_key) = event_data.get("registry_key") {
                        if let Some(key_str) = registry_key.as_str() {
                            for key in keys {
                                if let Some(expected_key) = key.as_str() {
                                    if key_str.to_lowercase().contains(&expected_key.to_lowercase()) {
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    async fn evaluate_auth_conditions(&self, conditions: &serde_json::Map<String, serde_json::Value>, event: &SecurityEvent) -> Result<bool> {
        // Check for multiple failed login attempts
        if let Some(count_threshold) = conditions.get("count") {
            if let Some(timeframe) = conditions.get("timeframe") {
                if let (Some(count), Some(seconds)) = (count_threshold.as_i64(), timeframe.as_i64()) {
                    return self.check_failed_login_pattern(event, count as i32, seconds as i32).await;
                }
            }
        }
        
        Ok(false)
    }
    
    async fn evaluate_generic_conditions(&self, conditions: &serde_json::Map<String, serde_json::Value>, event: &SecurityEvent) -> Result<bool> {
        // Generic rule evaluation for custom conditions
        
        // Check severity threshold
        if let Some(min_severity) = conditions.get("min_severity") {
            if let Some(severity_str) = min_severity.as_str() {
                let required_level = match severity_str {
                    "low" => 1,
                    "medium" => 2,
                    "high" => 3,
                    "critical" => 4,
                    _ => 1,
                };
                
                let event_level = match event.severity {
                    Severity::Low => 1,
                    Severity::Medium => 2,
                    Severity::High => 3,
                    Severity::Critical => 4,
                };
                
                if event_level >= required_level {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    async fn is_tor_exit_node(&self, ip: &str) -> Result<bool> {
        // Simplified TOR detection - in production, integrate with threat intelligence
        // Common TOR exit node patterns (this is a simplified example)
        let tor_patterns = ["127.0.0.1", "192.168.", "10.", "172."];
        for pattern in &tor_patterns {
            if ip.starts_with(pattern) {
                return Ok(false); // These are local IPs, not TOR
            }
        }
        
        // For demo purposes, flag any external IP as potentially suspicious
        Ok(!ip.starts_with("192.168.") && !ip.starts_with("10.") && !ip.starts_with("172."))
    }
    
    async fn is_known_malware_c2(&self, ip: &str) -> Result<bool> {
        // In production, this would check against threat intelligence feeds
        // For demo, we'll use a simple blacklist
        let known_malicious = ["198.51.100.", "203.0.113.", "192.0.2."];
        
        for malicious_prefix in &known_malicious {
            if ip.starts_with(malicious_prefix) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    async fn check_failed_login_pattern(&self, event: &SecurityEvent, count_threshold: i32, timeframe_seconds: i32) -> Result<bool> {
        // Check for multiple failed login attempts from the same source
        let failed_attempts = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM threats.security_events
            WHERE event_type = 'authentication'
                AND source_ip = $1
                AND event_data->>'success' = 'false'
                AND occurred_at >= NOW() - INTERVAL '1 second' * $2
            "#,
            event.source_ip,
            timeframe_seconds as f64
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .unwrap_or(0);
        
        Ok(failed_attempts >= count_threshold as i64)
    }

    pub async fn get_command_by_id(&self, command_id: Uuid) -> Result<Option<AgentCommand>> {
        let command = sqlx::query_as!(
            AgentCommand,
            r#"
            SELECT command_id, agent_id, issued_by, command_type, command_data,
                   status as "status: CommandStatus", result, issued_at, executed_at, completed_at
            FROM threats.agent_commands
            WHERE command_id = $1
            "#,
            command_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(command)
    }

    // Advanced threat analysis methods
    pub async fn analyze_threat_patterns(&self, agent_id: Uuid, hours: i32) -> Result<serde_json::Value> {
        let threat_summary = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_events,
                COUNT(CASE WHEN severity = 'critical' THEN 1 END) as critical_events,
                COUNT(CASE WHEN severity = 'high' THEN 1 END) as high_events,
                COUNT(CASE WHEN severity = 'medium' THEN 1 END) as medium_events,
                COUNT(CASE WHEN severity = 'low' THEN 1 END) as low_events,
                COUNT(DISTINCT event_type) as unique_event_types,
                array_agg(DISTINCT event_type) as event_types
            FROM threats.security_events
            WHERE agent_id = $1 
                AND occurred_at >= NOW() - INTERVAL '1 hour' * $2
            "#,
            agent_id,
            hours as f64
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let alert_summary = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_alerts,
                COUNT(CASE WHEN status = 'open' THEN 1 END) as open_alerts,
                COUNT(CASE WHEN severity = 'critical' THEN 1 END) as critical_alerts
            FROM threats.alerts
            WHERE agent_id = $1 
                AND created_at >= NOW() - INTERVAL '1 hour' * $2
            "#,
            agent_id,
            hours as f64
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(serde_json::json!({
            "agent_id": agent_id,
            "analysis_period_hours": hours,
            "threat_events": {
                "total": threat_summary.total_events.unwrap_or(0),
                "by_severity": {
                    "critical": threat_summary.critical_events.unwrap_or(0),
                    "high": threat_summary.high_events.unwrap_or(0),
                    "medium": threat_summary.medium_events.unwrap_or(0),
                    "low": threat_summary.low_events.unwrap_or(0)
                },
                "unique_event_types": threat_summary.unique_event_types.unwrap_or(0),
                "event_types": threat_summary.event_types.unwrap_or_default()
            },
            "alerts": {
                "total": alert_summary.total_alerts.unwrap_or(0),
                "open": alert_summary.open_alerts.unwrap_or(0),
                "critical": alert_summary.critical_alerts.unwrap_or(0)
            }
        }))
    }

    pub async fn get_threat_timeline(&self, agent_id: Option<Uuid>, hours: i32) -> Result<Vec<serde_json::Value>> {
        let timeline = if let Some(agent_id) = agent_id {
            sqlx::query_as!(
                TimelineRow,
                r#"
                SELECT 
                    DATE_TRUNC('hour', occurred_at) as hour,
                    event_type,
                    severity,
                    COUNT(*) as event_count
                FROM threats.security_events
                WHERE agent_id = $1 
                    AND occurred_at >= NOW() - INTERVAL '1 hour' * $2
                GROUP BY DATE_TRUNC('hour', occurred_at), event_type, severity
                ORDER BY hour DESC
                "#,
                agent_id,
                hours as f64
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query_as!(
                TimelineRow,
                r#"
                SELECT 
                    DATE_TRUNC('hour', occurred_at) as hour,
                    event_type,
                    severity,
                    COUNT(*) as event_count
                FROM threats.security_events
                WHERE occurred_at >= NOW() - INTERVAL '1 hour' * $1
                GROUP BY DATE_TRUNC('hour', occurred_at), event_type, severity
                ORDER BY hour DESC
                "#,
                hours as f64
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        };

        let timeline_data: Vec<serde_json::Value> = timeline
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "hour": row.hour,
                    "event_type": row.event_type,
                    "severity": row.severity,
                    "event_count": row.event_count.unwrap_or(0)
                })
            })
            .collect();

        Ok(timeline_data)
    }

    pub async fn create_custom_detection_rule(&self, name: String, rule_type: String, conditions: serde_json::Value, actions: serde_json::Value, severity: Severity, created_by: Uuid) -> Result<DetectionRule> {
        let rule = DetectionRule {
            rule_id: Uuid::new_v4(),
            name,
            description: Some(format!("Custom {} detection rule", rule_type)),
            rule_type,
            severity,
            conditions,
            actions,
            enabled: true,
            created_by: Some(created_by),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.create_detection_rule(rule, created_by).await
    }

    pub async fn bulk_process_events(&self, events: Vec<(Uuid, CreateSecurityEventRequest)>) -> Result<Vec<SecurityEvent>> {
        let mut processed_events = Vec::new();
        
        for (agent_id, event_request) in events {
            match self.create_security_event(agent_id, event_request).await {
                Ok(event) => processed_events.push(event),
                Err(e) => {
                    tracing::warn!("Failed to process event for agent {}: {}", agent_id, e);
                    continue;
                }
            }
        }
        
        Ok(processed_events)
    }

    pub async fn get_top_threats(&self, hours: i32, limit: i32) -> Result<Vec<serde_json::Value>> {
        let top_threats = sqlx::query!(
            r#"
            SELECT 
                event_type,
                severity,
                COUNT(*) as threat_count,
                COUNT(DISTINCT agent_id) as affected_agents,
                MAX(occurred_at) as latest_occurrence
            FROM threats.security_events
            WHERE occurred_at >= NOW() - INTERVAL '1 hour' * $1
            GROUP BY event_type, severity
            ORDER BY 
                CASE severity 
                    WHEN 'critical' THEN 4
                    WHEN 'high' THEN 3
                    WHEN 'medium' THEN 2
                    WHEN 'low' THEN 1
                END DESC,
                threat_count DESC
            LIMIT $2
            "#,
            hours as f64,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let threat_data: Vec<serde_json::Value> = top_threats
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "event_type": row.event_type,
                    "severity": row.severity,
                    "threat_count": row.threat_count.unwrap_or(0),
                    "affected_agents": row.affected_agents.unwrap_or(0),
                    "latest_occurrence": row.latest_occurrence
                })
            })
            .collect();

        Ok(threat_data)
    }
}