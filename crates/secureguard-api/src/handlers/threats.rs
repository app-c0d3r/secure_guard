use crate::{
    database::Database, middleware::auth::AuthUser, services::threat_service::ThreatService,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use secureguard_shared::{
    AgentCommand, AlertStatus, CommandStatus, CreateAlertRequest, CreateCommandRequest,
    CreateSecurityEventRequest, DetectionRule, SecureGuardError, SecurityEvent, Severity,
    ThreatAlert, UpdateAlertRequest,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub agent_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub severity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    pub agent_id: Option<Uuid>,
    pub status: Option<String>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AnalysisQuery {
    pub hours: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    pub agent_id: Option<Uuid>,
    pub hours: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TopThreatsQuery {
    pub hours: Option<i32>,
    pub limit: Option<i32>,
}

// Security Events Endpoints
pub async fn create_security_event(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<CreateSecurityEventRequest>,
) -> Result<(StatusCode, Json<SecurityEvent>), (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let event = threat_service
        .create_security_event(agent_id, request)
        .await
        .map_err(|e| handle_error(e))?;

    Ok((StatusCode::CREATED, Json(event)))
}

pub async fn get_security_events(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Query(params): Query<EventsQuery>,
) -> Result<Json<Vec<SecurityEvent>>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let events = threat_service
        .get_security_events(params.agent_id, params.limit)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(events))
}

// Detection Rules Endpoints
pub async fn create_detection_rule(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Json(rule): Json<DetectionRule>,
) -> Result<(StatusCode, Json<DetectionRule>), (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let created_rule = threat_service
        .create_detection_rule(rule, user.user_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok((StatusCode::CREATED, Json(created_rule)))
}

pub async fn get_detection_rules(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Query(params): Query<EnabledOnlyQuery>,
) -> Result<Json<Vec<DetectionRule>>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let rules = threat_service
        .get_detection_rules(params.enabled_only.unwrap_or(false))
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(rules))
}

#[derive(Debug, Deserialize)]
pub struct EnabledOnlyQuery {
    pub enabled_only: Option<bool>,
}

// Threat Alerts Endpoints
pub async fn create_alert(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<CreateAlertRequest>,
) -> Result<(StatusCode, Json<ThreatAlert>), (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let alert = threat_service
        .create_alert(agent_id, request)
        .await
        .map_err(|e| handle_error(e))?;

    Ok((StatusCode::CREATED, Json(alert)))
}

pub async fn update_alert(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Path(alert_id): Path<Uuid>,
    Json(request): Json<UpdateAlertRequest>,
) -> Result<Json<ThreatAlert>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let alert = threat_service
        .update_alert(alert_id, request)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(alert))
}

pub async fn get_alerts(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Query(params): Query<AlertsQuery>,
) -> Result<Json<Vec<ThreatAlert>>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let status = params.status.and_then(|s| match s.as_str() {
        "open" => Some(AlertStatus::Open),
        "investigating" => Some(AlertStatus::Investigating),
        "resolved" => Some(AlertStatus::Resolved),
        "false_positive" => Some(AlertStatus::FalsePositive),
        _ => None,
    });

    let alerts = threat_service
        .get_alerts(params.agent_id, status)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(alerts))
}

// Agent Commands Endpoints
pub async fn create_agent_command(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<CreateCommandRequest>,
) -> Result<(StatusCode, Json<AgentCommand>), (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let command = threat_service
        .create_command(agent_id, user.user_id, request)
        .await
        .map_err(|e| handle_error(e))?;

    // TODO: Send command to agent via WebSocket
    // This would integrate with the WebSocket system to actually send the command

    Ok((StatusCode::CREATED, Json(command)))
}

pub async fn get_agent_commands(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Vec<AgentCommand>>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    // Get commands for specific agent
    // TODO: Implement get_commands_for_agent method in ThreatService
    let commands: Vec<AgentCommand> = vec![]; // Placeholder

    Ok(Json(commands))
}

pub async fn update_command_status(
    State(db): State<Database>,
    Path(command_id): Path<Uuid>,
    Json(status_update): Json<CommandStatusUpdate>,
) -> Result<Json<AgentCommand>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let command = threat_service
        .update_command_status(command_id, status_update.status, status_update.result)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(command))
}

#[derive(Debug, Deserialize)]
pub struct CommandStatusUpdate {
    pub status: CommandStatus,
    pub result: Option<serde_json::Value>,
}

// Threat Analytics Endpoints
pub async fn get_threat_summary(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
) -> Result<Json<ThreatSummary>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    // Get recent alerts and events for summary
    let recent_alerts = threat_service
        .get_alerts(None, None)
        .await
        .map_err(|e| handle_error(e))?;

    let recent_events = threat_service
        .get_security_events(None, Some(100))
        .await
        .map_err(|e| handle_error(e))?;

    let summary = ThreatSummary {
        total_alerts: recent_alerts.len(),
        open_alerts: recent_alerts
            .iter()
            .filter(|a| matches!(a.status, AlertStatus::Open))
            .count(),
        critical_alerts: recent_alerts
            .iter()
            .filter(|a| matches!(a.severity, Severity::Critical))
            .count(),
        total_events_today: recent_events
            .iter()
            .filter(|e| e.occurred_at.date_naive() == chrono::Utc::now().date_naive())
            .count(),
        top_threat_types: get_top_threat_types(&recent_alerts),
        agent_threat_distribution: get_agent_threat_distribution(&recent_alerts),
    };

    Ok(Json(summary))
}

#[derive(Debug, serde::Serialize)]
pub struct ThreatSummary {
    pub total_alerts: usize,
    pub open_alerts: usize,
    pub critical_alerts: usize,
    pub total_events_today: usize,
    pub top_threat_types: Vec<ThreatTypeCount>,
    pub agent_threat_distribution: Vec<AgentThreatCount>,
}

#[derive(Debug, serde::Serialize)]
pub struct ThreatTypeCount {
    pub threat_type: String,
    pub count: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct AgentThreatCount {
    pub agent_id: Uuid,
    pub threat_count: usize,
}

fn get_top_threat_types(alerts: &[ThreatAlert]) -> Vec<ThreatTypeCount> {
    use std::collections::HashMap;

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for alert in alerts {
        *type_counts.entry(alert.alert_type.clone()).or_insert(0) += 1;
    }

    let mut counts: Vec<_> = type_counts
        .into_iter()
        .map(|(threat_type, count)| ThreatTypeCount { threat_type, count })
        .collect();

    counts.sort_by(|a, b| b.count.cmp(&a.count));
    counts.truncate(5); // Top 5
    counts
}

fn get_agent_threat_distribution(alerts: &[ThreatAlert]) -> Vec<AgentThreatCount> {
    use std::collections::HashMap;

    let mut agent_counts: HashMap<Uuid, usize> = HashMap::new();
    for alert in alerts {
        *agent_counts.entry(alert.agent_id).or_insert(0) += 1;
    }

    let mut counts: Vec<_> = agent_counts
        .into_iter()
        .map(|(agent_id, threat_count)| AgentThreatCount {
            agent_id,
            threat_count,
        })
        .collect();

    counts.sort_by(|a, b| b.threat_count.cmp(&a.threat_count));
    counts.truncate(10); // Top 10
    counts
}

// Advanced Threat Analysis Endpoints
pub async fn analyze_threat_patterns(
    State(db): State<Database>,
    AuthUser(_user): AuthUser,
    Path(agent_id): Path<Uuid>,
    Query(query): Query<AnalysisQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());
    let hours = query.hours.unwrap_or(24);

    let analysis = threat_service
        .analyze_threat_patterns(agent_id, hours)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(analysis))
}

pub async fn get_threat_timeline(
    State(db): State<Database>,
    AuthUser(_user): AuthUser,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());
    let hours = query.hours.unwrap_or(24);

    let timeline = threat_service
        .get_threat_timeline(query.agent_id, hours)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(timeline))
}

pub async fn get_top_threats(
    State(db): State<Database>,
    AuthUser(_user): AuthUser,
    Query(query): Query<TopThreatsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());
    let hours = query.hours.unwrap_or(24);
    let limit = query.limit.unwrap_or(10);

    let top_threats = threat_service
        .get_top_threats(hours, limit)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(top_threats))
}

pub async fn bulk_create_events(
    State(db): State<Database>,
    AuthUser(_user): AuthUser,
    Json(events): Json<Vec<(Uuid, CreateSecurityEventRequest)>>,
) -> Result<(StatusCode, Json<Vec<SecurityEvent>>), (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let processed_events = threat_service
        .bulk_process_events(events)
        .await
        .map_err(|e| handle_error(e))?;

    Ok((StatusCode::CREATED, Json(processed_events)))
}

pub async fn create_command(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<CreateCommandRequest>,
) -> Result<(StatusCode, Json<AgentCommand>), (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let command = threat_service
        .create_command(agent_id, user.user_id, request)
        .await
        .map_err(|e| handle_error(e))?;

    Ok((StatusCode::CREATED, Json(command)))
}

pub async fn get_command(
    State(db): State<Database>,
    AuthUser(_user): AuthUser,
    Path(command_id): Path<Uuid>,
) -> Result<Json<Option<AgentCommand>>, (StatusCode, Json<serde_json::Value>)> {
    let threat_service = ThreatService::new(db.pool().clone());

    let command = threat_service
        .get_command_by_id(command_id)
        .await
        .map_err(|e| handle_error(e))?;

    Ok(Json(command))
}

fn handle_error(error: SecureGuardError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, message) = match error {
        SecureGuardError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        SecureGuardError::AgentNotFound => (StatusCode::NOT_FOUND, "Agent not found".to_string()),
        SecureGuardError::DatabaseError(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        ),
    };

    (status, Json(serde_json::json!({ "error": message })))
}
