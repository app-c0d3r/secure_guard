use axum::{
    extract::{ws::{WebSocket, Message}, State, Query},
    response::Response,
    http::StatusCode,
};
use axum::extract::ws::WebSocketUpgrade;
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;
use serde::Deserialize;
use serde_json;
use tracing::{info, warn, error, debug};

use crate::{
    database::Database,
    websocket::connection_manager::ConnectionManager,
    services::{user_service::UserService, agent_service::AgentService, threat_service::ThreatService, auth_service::AuthService},
    middleware::auth::AuthUser,
};
use secureguard_shared::{DashboardMessage, AgentStatus};

#[derive(Debug, Deserialize)]
pub struct DashboardWebSocketQuery {
    pub token: String,
}

pub async fn handle_dashboard_websocket(
    ws: WebSocketUpgrade,
    Query(params): Query<DashboardWebSocketQuery>,
    State(db): State<Database>,
    State(connection_manager): State<ConnectionManager>,
) -> Result<Response, StatusCode> {
    // Authenticate user from token
    let auth_service = AuthService::new("your-secret-key-change-in-production".to_string());
    
    let claims = match auth_service.verify_token(&params.token) {
        Ok(claims) => claims,
        Err(_) => {
            warn!("Invalid token for dashboard WebSocket connection");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(user_id) => user_id,
        Err(_) => {
            warn!("Invalid user ID in token for dashboard WebSocket");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    // Verify user exists
    let user_service = UserService::new(db.pool().clone(), auth_service);
    match user_service.find_by_id(user_id).await {
        Ok(Some(_)) => {
            info!("User {} authenticated for dashboard WebSocket", user_id);
            Ok(ws.on_upgrade(move |socket| {
                handle_dashboard_socket(socket, user_id, db, connection_manager)
            }))
        }
        Ok(None) => {
            warn!("User {} not found for dashboard WebSocket", user_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Database error checking user {}: {}", user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_dashboard_socket(
    socket: WebSocket,
    user_id: Uuid,
    db: Database,
    connection_manager: ConnectionManager,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    
    // Register the dashboard connection
    let connection_id = connection_manager.add_dashboard_connection(user_id, tx).await;
    
    // Send initial dashboard data
    if let Err(e) = send_initial_dashboard_data(&connection_manager, user_id, &db).await {
        error!("Failed to send initial dashboard data to user {}: {}", user_id, e);
    }
    
    // Task to handle outgoing messages to the dashboard
    let outgoing_task = {
        let connection_manager = connection_manager.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = sender.send(Message::Text(msg)).await {
                    error!("Failed to send message to dashboard user {}: {}", user_id, e);
                    break;
                }
            }
            
            // Clean up connection when sender task ends
            connection_manager.remove_connection(connection_id).await;
        })
    };
    
    // Task to handle incoming messages from the dashboard
    let incoming_task = {
        let db = db.clone();
        let connection_manager = connection_manager.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        debug!("Received message from dashboard user {}: {}", user_id, text);
                        
                        // Handle dashboard requests (e.g., agent commands, data requests)
                        if let Err(e) = handle_dashboard_message(user_id, &text, &db, &connection_manager).await {
                            error!("Error handling dashboard message: {}", e);
                        }
                    }
                    Ok(Message::Binary(_)) => {
                        debug!("Received binary data from dashboard user {}", user_id);
                        // Handle binary data if needed
                    }
                    Ok(Message::Ping(_)) => {
                        debug!("Received ping from dashboard user {}", user_id);
                    }
                    Ok(Message::Pong(_)) => {
                        debug!("Received pong from dashboard user {}", user_id);
                    }
                    Ok(Message::Close(_)) => {
                        info!("Dashboard user {} closed WebSocket connection", user_id);
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error with dashboard user {}: {}", user_id, e);
                        break;
                    }
                }
            }
            
            info!("Dashboard user {} WebSocket handler finished", user_id);
        })
    };
    
    // Wait for either task to complete
    tokio::select! {
        _ = outgoing_task => {
            info!("Dashboard user {} outgoing task completed", user_id);
        }
        _ = incoming_task => {
            info!("Dashboard user {} incoming task completed", user_id);
        }
    }
    
    // Ensure connection is cleaned up
    connection_manager.remove_connection(connection_id).await;
    info!("Dashboard user {} WebSocket connection fully closed", user_id);
}

async fn send_initial_dashboard_data(
    connection_manager: &ConnectionManager,
    user_id: Uuid,
    db: &Database,
) -> Result<(), String> {
    let agent_service = AgentService::new(db.pool().clone());
    let threat_service = ThreatService::new(db.pool().clone());
    
    // Send current agent statuses
    let agents = agent_service.list_agents_for_tenant(Uuid::new_v4()).await // TODO: Use actual tenant
        .map_err(|e| format!("Failed to get agents: {}", e))?;
    
    for agent in agents {
        let message = DashboardMessage::AgentStatusUpdate {
            agent_id: agent.agent_id,
            status: agent.status,
            last_seen: agent.last_heartbeat.unwrap_or(agent.created_at),
        };
        
        connection_manager.send_to_user_dashboards(user_id, &message).await
            .map_err(|e| format!("Failed to send agent status: {}", e))?;
    }
    
    // Send recent alerts
    let recent_alerts = threat_service.get_alerts(None, None).await
        .map_err(|e| format!("Failed to get alerts: {}", e))?;
    
    for alert in recent_alerts.into_iter().take(10) { // Send last 10 alerts
        let message = DashboardMessage::NewThreatAlert {
            alert: alert.clone(),
            agent_name: format!("Agent-{}", alert.agent_id),
            event_title: alert.title.clone(),
        };
        
        connection_manager.send_to_user_dashboards(user_id, &message).await
            .map_err(|e| format!("Failed to send alert: {}", e))?;
    }
    
    info!("Sent initial dashboard data to user {}", user_id);
    Ok(())
}

async fn handle_dashboard_message(
    user_id: Uuid,
    message: &str,
    db: &Database,
    connection_manager: &ConnectionManager,
) -> Result<(), String> {
    // Parse dashboard request
    let request: serde_json::Value = serde_json::from_str(message)
        .map_err(|e| format!("Failed to parse dashboard message: {}", e))?;
    
    match request.get("type").and_then(|t| t.as_str()) {
        Some("agent_command") => {
            handle_agent_command_request(user_id, &request, db, connection_manager).await
        }
        Some("refresh_data") => {
            send_initial_dashboard_data(connection_manager, user_id, db).await
        }
        Some("ping") => {
            // Respond to ping with pong
            let pong_message = DashboardMessage::AgentStatusUpdate {
                agent_id: Uuid::new_v4(), // Dummy data for ping response
                status: AgentStatus::Online,
                last_seen: chrono::Utc::now(),
            };
            connection_manager.send_to_user_dashboards(user_id, &pong_message).await
        }
        _ => {
            warn!("Unknown dashboard message type from user {}: {:?}", user_id, request.get("type"));
            Ok(())
        }
    }
}

async fn handle_agent_command_request(
    user_id: Uuid,
    request: &serde_json::Value,
    db: &Database,
    connection_manager: &ConnectionManager,
) -> Result<(), String> {
    let agent_id_str = request.get("agent_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing agent_id in command request")?;
    
    let agent_id = Uuid::parse_str(agent_id_str)
        .map_err(|e| format!("Invalid agent_id: {}", e))?;
    
    let command_type = request.get("command_type")
        .and_then(|v| v.as_str())
        .ok_or("Missing command_type in command request")?;
    
    let command_data = request.get("command_data")
        .unwrap_or(&serde_json::json!({}))
        .clone();
    
    // Create command in database
    let threat_service = ThreatService::new(db.pool().clone());
    let command_request = secureguard_shared::CreateCommandRequest {
        command_type: command_type.to_string(),
        command_data,
    };
    
    let command = threat_service.create_command(agent_id, user_id, command_request).await
        .map_err(|e| format!("Failed to create command: {}", e))?;
    
    // Send command to agent via WebSocket
    let command_type_for_log = command.command_type.clone();
    crate::websocket::agent_handler::send_command_to_agent(
        agent_id,
        command.command_id,
        command.command_type,
        command.command_data,
        connection_manager,
    ).await
        .map_err(|e| format!("Failed to send command to agent: {}", e))?;
    
    info!("User {} sent command {} to agent {}", user_id, command_type_for_log, agent_id);
    Ok(())
}

// Helper function for broadcasting dashboard messages
pub async fn broadcast_dashboard_message(
    connection_manager: &ConnectionManager,
    message: &DashboardMessage,
) -> Result<(), String> {
    connection_manager.send_to_all_dashboards(message).await
}