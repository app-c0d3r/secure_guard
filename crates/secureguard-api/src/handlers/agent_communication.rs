use axum::{
    extract::{ws::WebSocket, Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::Response,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    database::Database,
    middleware::auth::AuthUser,
    services::agent_communication::{
        AgentCommunicationService, AgentConnection, CommunicationStats,
    },
};
use secureguard_shared::{
    Agent, AgentCommand, AgentHeartbeat, AgentMessage, AgentStatus, CommandStatus,
    CreateAgentRequest, CreateSecurityEventRequest, SecureGuardError,
};

#[derive(Debug, Deserialize)]
pub struct AgentQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command_type: String,
    pub command_data: serde_json::Value,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct BulkCommandRequest {
    pub agent_ids: Vec<Uuid>,
    pub command_type: String,
    pub command_data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CommandResponseRequest {
    pub command_id: Uuid,
    pub status: CommandStatus,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct EmergencyIsolationRequest {
    pub agent_ids: Vec<Uuid>,
    pub reason: String,
    pub isolation_level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentStatusResponse {
    pub agent: Agent,
    pub connection: Option<AgentConnection>,
    pub pending_commands: usize,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct CommunicationOverview {
    pub stats: CommunicationStats,
    pub recent_activity: Vec<ActivityEvent>,
    pub system_health: SystemHealth,
}

#[derive(Debug, Serialize)]
pub struct ActivityEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub agent_id: Uuid,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub overall_status: String,
    pub response_time_ms: f64,
    pub error_rate_percent: f64,
    pub uptime_hours: f64,
}

// Agent Registration and Management
pub async fn register_agent_enhanced(
    State(db): State<Database>,
    ws: WebSocketUpgrade,
    Json(request): Json<CreateAgentRequest>,
) -> Response {
    info!(
        "Enhanced agent registration request from: {}",
        request.hostname
    );

    ws.on_upgrade(move |socket| handle_agent_websocket(socket, db, request))
}

pub async fn handle_agent_websocket(
    websocket: WebSocket,
    db: Database,
    registration_request: CreateAgentRequest,
) {
    let connection_id = Uuid::new_v4();
    let (mut sender, mut receiver) = websocket.split();

    // In a real implementation, we'd get the communication service from app state
    // For now, we'll simulate the registration process
    info!(
        "Agent WebSocket connection established: {} ({})",
        registration_request.agent_name, connection_id
    );

    // Send registration confirmation
    let confirmation = AgentMessage::RegistrationConfirmed {
        agent_id: Uuid::new_v4(),
        configuration: serde_json::json!({
            "reporting_interval": 30,
            "compression_enabled": true,
            "encryption_level": "AES256"
        }),
    };

    if let Ok(message) = serde_json::to_string(&confirmation) {
        if let Err(e) = sender.send(axum::extract::ws::Message::Text(message)).await {
            error!("Failed to send registration confirmation: {}", e);
            return;
        }
    }

    // Handle incoming messages from agent
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                match serde_json::from_str::<AgentMessage>(&text) {
                    Ok(agent_msg) => {
                        if let Err(e) = handle_agent_message(agent_msg, &mut sender).await {
                            error!("Failed to handle agent message: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse agent message: {}", e);
                    }
                }
            }
            Ok(axum::extract::ws::Message::Binary(_)) => {
                debug!("Received binary message from agent (not supported yet)");
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                info!("Agent WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    info!("Agent WebSocket connection terminated: {}", connection_id);
}

async fn handle_agent_message(
    message: AgentMessage,
    sender: &mut futures_util::stream::SplitSink<WebSocket, axum::extract::ws::Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match message {
        AgentMessage::Heartbeat {
            agent_id,
            status,
            metrics,
        } => {
            debug!("Received heartbeat from agent: {}", agent_id);

            // Send heartbeat acknowledgment
            let ack = AgentMessage::HeartbeatAck {
                timestamp: chrono::Utc::now(),
            };
            let response = serde_json::to_string(&ack)?;
            sender
                .send(axum::extract::ws::Message::Text(response))
                .await?;
        }

        AgentMessage::SecurityEvents { agent_id, events } => {
            info!(
                "Received {} security events from agent: {}",
                events.len(),
                agent_id
            );

            // Process events (in real implementation, this would go through the communication service)
            let ack = AgentMessage::EventsProcessed {
                processed_count: events.len(),
                timestamp: chrono::Utc::now(),
            };
            let response = serde_json::to_string(&ack)?;
            sender
                .send(axum::extract::ws::Message::Text(response))
                .await?;
        }

        AgentMessage::CommandResponse {
            command_id,
            status,
            result,
        } => {
            info!("Received command response: {} -> {:?}", command_id, status);

            // Process command response
            // In real implementation, this would update the command tracking system
        }

        _ => {
            debug!("Received unhandled agent message type");
        }
    }

    Ok(())
}

// Agent Status and Monitoring
pub async fn get_agent_status_detailed(
    State(db): State<Database>,
    AuthUser(_user): AuthUser,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<AgentStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    // In a real implementation, we'd use the communication service
    let mock_response = AgentStatusResponse {
        agent: Agent {
            agent_id,
            tenant_id: Uuid::new_v4(),
            hardware_fingerprint: "HW123456789".to_string(),
            os_info: serde_json::json!({
                "hostname": "WIN-DESKTOP-001",
                "os": "Windows 10 Pro",
                "ip_address": "192.168.1.100",
                "mac_address": "00:11:22:33:44:55"
            }),
            status: AgentStatus::Online,
            last_heartbeat: Some(chrono::Utc::now()),
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now() - chrono::Duration::hours(24),
            user_id: Some(Uuid::new_v4()),
            device_name: Some("WIN-DESKTOP-001".to_string()),
            registered_via_key_id: None,
            registered_via_token_id: None,
        },
        connection: Some(AgentConnection {
            agent_id,
            connection_id: Uuid::new_v4(),
            last_heartbeat: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
            status: AgentStatus::Online,
            version: "1.0.0".to_string(),
            capabilities: vec![
                "file_monitoring".to_string(),
                "process_monitoring".to_string(),
            ],
            pending_commands: vec![],
            metrics_buffer: vec![],
        }),
        pending_commands: 0,
        last_activity: chrono::Utc::now(),
    };

    Ok(Json(mock_response))
}

pub async fn list_agents_with_status(
    State(db): State<Database>,
    AuthUser(_user): AuthUser,
    Query(query): Query<AgentQuery>,
) -> Result<Json<Vec<AgentStatusResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let limit = query.limit.unwrap_or(50);

    // Generate mock agent list
    let agents: Vec<_> = (0..limit.min(10))
        .map(|i| {
            let agent_id = Uuid::new_v4();
            AgentStatusResponse {
                agent: Agent {
                    agent_id,
                    tenant_id: Uuid::new_v4(),
                    hardware_fingerprint: format!("HW{:09}", 123456789 + i),
                    os_info: serde_json::json!({
                        "hostname": format!("WIN-DESKTOP-{:03}", i + 1),
                        "os": "Windows 10 Pro",
                        "ip_address": format!("192.168.1.{}", 100 + i),
                        "mac_address": format!("00:11:22:33:44:{:02X}", 55 + i)
                    }),
                    status: if i % 4 == 0 {
                        AgentStatus::Offline
                    } else {
                        AgentStatus::Online
                    },
                    last_heartbeat: Some(
                        chrono::Utc::now() - chrono::Duration::minutes((i * 10) as i64),
                    ),
                    version: "1.0.0".to_string(),
                    created_at: chrono::Utc::now() - chrono::Duration::days(i as i64),
                    user_id: Some(Uuid::new_v4()),
                    device_name: Some(format!("WIN-DESKTOP-{:03}", i + 1)),
                    registered_via_key_id: None,
                    registered_via_token_id: None,
                },
                connection: if i % 4 != 0 {
                    Some(AgentConnection {
                        agent_id,
                        connection_id: Uuid::new_v4(),
                        last_heartbeat: chrono::Utc::now()
                            - chrono::Duration::minutes((i * 2) as i64),
                        last_seen: chrono::Utc::now() - chrono::Duration::minutes(i as i64),
                        status: AgentStatus::Online,
                        version: "1.0.0".to_string(),
                        capabilities: vec![
                            "file_monitoring".to_string(),
                            "process_monitoring".to_string(),
                        ],
                        pending_commands: if i % 3 == 0 {
                            vec![Uuid::new_v4()]
                        } else {
                            vec![]
                        },
                        metrics_buffer: vec![],
                    })
                } else {
                    None
                },
                pending_commands: if i % 3 == 0 { 1 } else { 0 },
                last_activity: chrono::Utc::now() - chrono::Duration::minutes((i * 2) as i64),
            }
        })
        .collect();

    Ok(Json(agents))
}

// Command Distribution and Management
pub async fn send_command_to_agent(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<CommandRequest>,
) -> Result<Json<AgentCommand>, (StatusCode, Json<serde_json::Value>)> {
    let command = AgentCommand {
        command_id: Uuid::new_v4(),
        agent_id,
        issued_by: user.user_id,
        command_type: request.command_type,
        command_data: request.command_data,
        status: CommandStatus::Pending,
        result: None,
        issued_at: chrono::Utc::now(),
        executed_at: None,
        completed_at: None,
    };

    // In real implementation, this would go through the communication service
    info!(
        "Command issued to agent {}: {} by user {}",
        agent_id, command.command_type, user.user_id
    );

    Ok(Json(command))
}

pub async fn send_bulk_command(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Json(request): Json<BulkCommandRequest>,
) -> Result<Json<Vec<AgentCommand>>, (StatusCode, Json<serde_json::Value>)> {
    if request.agent_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No agents specified"})),
        ));
    }

    if request.agent_ids.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Too many agents specified (max 100)"})),
        ));
    }

    let commands: Vec<_> = request
        .agent_ids
        .into_iter()
        .map(|agent_id| AgentCommand {
            command_id: Uuid::new_v4(),
            agent_id,
            issued_by: user.user_id,
            command_type: request.command_type.clone(),
            command_data: request.command_data.clone(),
            status: CommandStatus::Pending,
            result: None,
            issued_at: chrono::Utc::now(),
            executed_at: None,
            completed_at: None,
        })
        .collect();

    info!(
        "Bulk command issued to {} agents: {} by user {}",
        commands.len(),
        request.command_type,
        user.user_id
    );

    Ok(Json(commands))
}

// Emergency Operations
pub async fn emergency_isolate_agents(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Json(request): Json<EmergencyIsolationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if request.agent_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No agents specified for isolation"})),
        ));
    }

    warn!(
        "EMERGENCY ISOLATION requested by user {} for {} agents: {}",
        user.user_id,
        request.agent_ids.len(),
        request.reason
    );

    // In real implementation, this would use the communication service
    let response = serde_json::json!({
        "action": "emergency_isolation",
        "initiated_by": user.user_id,
        "agents_targeted": request.agent_ids,
        "reason": request.reason,
        "isolation_level": request.isolation_level.unwrap_or_else(|| "full".to_string()),
        "timestamp": chrono::Utc::now(),
        "status": "initiated",
        "estimated_completion": chrono::Utc::now() + chrono::Duration::seconds(30)
    });

    Ok(Json(response))
}

pub async fn broadcast_emergency_command(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Json(request): Json<CommandRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    warn!(
        "EMERGENCY BROADCAST requested by user {}: {}",
        user.user_id, request.command_type
    );

    let response = serde_json::json!({
        "action": "emergency_broadcast",
        "initiated_by": user.user_id,
        "command_type": request.command_type,
        "broadcast_scope": "all_online_agents",
        "estimated_targets": 47, // Mock number
        "timestamp": chrono::Utc::now(),
        "status": "broadcasting"
    });

    Ok(Json(response))
}

// Communication Statistics and Analytics
pub async fn get_communication_overview(
    State(db): State<Database>,
    AuthUser(_user): AuthUser,
) -> Result<Json<CommunicationOverview>, (StatusCode, Json<serde_json::Value>)> {
    let mock_stats = CommunicationStats {
        total_agents: 47,
        online_agents: 43,
        offline_agents: 4,
        commands_sent_24h: 156,
        commands_completed_24h: 142,
        events_received_24h: 15647,
        average_response_time_ms: 245.7,
        heartbeat_failures: 3,
    };

    let recent_activity = vec![
        ActivityEvent {
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(5),
            event_type: "agent_registered".to_string(),
            agent_id: Uuid::new_v4(),
            description: "New agent WIN-DESKTOP-048 registered".to_string(),
        },
        ActivityEvent {
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(12),
            event_type: "command_completed".to_string(),
            agent_id: Uuid::new_v4(),
            description: "Update configuration command completed".to_string(),
        },
        ActivityEvent {
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(18),
            event_type: "agent_offline".to_string(),
            agent_id: Uuid::new_v4(),
            description: "Agent WIN-DESKTOP-023 went offline (heartbeat timeout)".to_string(),
        },
    ];

    let system_health = SystemHealth {
        overall_status: "healthy".to_string(),
        response_time_ms: 245.7,
        error_rate_percent: 0.8,
        uptime_hours: 127.3,
    };

    let overview = CommunicationOverview {
        stats: mock_stats,
        recent_activity,
        system_health,
    };

    Ok(Json(overview))
}

pub async fn get_agent_performance_metrics(
    State(db): State<Database>,
    AuthUser(_user): AuthUser,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let metrics = serde_json::json!({
        "agent_id": agent_id,
        "performance_metrics": {
            "cpu_usage_percent": 15.4,
            "memory_usage_mb": 245,
            "disk_usage_percent": 67.8,
            "network_io": {
                "bytes_sent_24h": 1048576,
                "bytes_received_24h": 2097152
            }
        },
        "communication_metrics": {
            "heartbeats_sent_24h": 2880,
            "heartbeat_success_rate": 99.9,
            "events_sent_24h": 1247,
            "commands_received_24h": 7,
            "average_response_time_ms": 123.4
        },
        "security_metrics": {
            "threats_detected_24h": 23,
            "high_severity_events": 2,
            "blocked_actions": 15
        },
        "timestamp": chrono::Utc::now()
    });

    Ok(Json(metrics))
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
