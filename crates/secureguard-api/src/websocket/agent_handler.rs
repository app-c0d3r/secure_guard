use axum::extract::ws::WebSocketUpgrade;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State,
    },
    http::StatusCode,
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use serde_json;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    database::Database,
    services::{agent_service::AgentService, threat_service::ThreatService},
    websocket::connection_manager::ConnectionManager,
};
use secureguard_shared::{
    AgentMessage, AgentStatus, CommandResponse, CreateSecurityEventRequest, DashboardMessage,
    Severity,
};

#[derive(Debug, Deserialize)]
pub struct AgentWebSocketQuery {
    pub agent_id: Uuid,
    pub token: Option<String>,
}

pub async fn handle_agent_websocket(
    ws: WebSocketUpgrade,
    Query(params): Query<AgentWebSocketQuery>,
    State(db): State<Database>,
    State(connection_manager): State<ConnectionManager>,
) -> Result<Response, StatusCode> {
    // TODO: Implement proper agent authentication
    // For now, we'll use a simple token check or skip authentication in development

    info!("Agent {} requesting WebSocket connection", params.agent_id);

    // Verify agent exists in database
    let agent_service = AgentService::new(db.pool().clone());
    match agent_service.find_by_id(params.agent_id).await {
        Ok(Some(_)) => {
            info!("Agent {} authenticated for WebSocket", params.agent_id);
            Ok(ws.on_upgrade(move |socket| {
                handle_agent_socket(socket, params.agent_id, db, connection_manager)
            }))
        }
        Ok(None) => {
            warn!("Agent {} not found in database", params.agent_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Database error checking agent {}: {}", params.agent_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_agent_socket(
    socket: WebSocket,
    agent_id: Uuid,
    db: Database,
    connection_manager: ConnectionManager,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Register the connection
    let connection_id = connection_manager.add_agent_connection(agent_id, tx).await;

    // Task to handle outgoing messages to the agent
    let outgoing_task = {
        let connection_manager = connection_manager.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = sender.send(Message::Text(msg)).await {
                    error!("Failed to send message to agent {}: {}", agent_id, e);
                    break;
                }
            }

            // Clean up connection when sender task ends
            connection_manager.remove_connection(connection_id).await;
        })
    };

    // Task to handle incoming messages from the agent
    let incoming_task = {
        let db = db.clone();
        let connection_manager = connection_manager.clone();

        tokio::spawn(async move {
            let agent_service = AgentService::new(db.pool().clone());
            let threat_service = ThreatService::new(db.pool().clone());

            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        debug!("Received message from agent {}: {}", agent_id, text);

                        match serde_json::from_str::<AgentMessage>(&text) {
                            Ok(agent_message) => {
                                if let Err(e) = handle_agent_message(
                                    agent_id,
                                    agent_message,
                                    &agent_service,
                                    &threat_service,
                                    &connection_manager,
                                )
                                .await
                                {
                                    error!("Error handling agent message: {}", e);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse message from agent {}: {}", agent_id, e);
                            }
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        debug!(
                            "Received binary data from agent {} ({} bytes)",
                            agent_id,
                            data.len()
                        );
                        // Handle binary data if needed (e.g., file uploads, compressed data)
                    }
                    Ok(Message::Ping(_)) => {
                        debug!("Received ping from agent {}", agent_id);
                        // WebSocket will automatically handle pong
                    }
                    Ok(Message::Pong(_)) => {
                        debug!("Received pong from agent {}", agent_id);
                    }
                    Ok(Message::Close(_)) => {
                        info!("Agent {} closed WebSocket connection", agent_id);
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error with agent {}: {}", agent_id, e);
                        break;
                    }
                }
            }

            info!("Agent {} WebSocket handler finished", agent_id);
        })
    };

    // Wait for either task to complete
    tokio::select! {
        _ = outgoing_task => {
            info!("Agent {} outgoing task completed", agent_id);
        }
        _ = incoming_task => {
            info!("Agent {} incoming task completed", agent_id);
        }
    }

    // Ensure connection is cleaned up
    connection_manager.remove_connection(connection_id).await;
    info!("Agent {} WebSocket connection fully closed", agent_id);
}

async fn handle_agent_message(
    agent_id: Uuid,
    message: AgentMessage,
    agent_service: &AgentService,
    threat_service: &ThreatService,
    connection_manager: &ConnectionManager,
) -> Result<(), String> {
    match message {
        AgentMessage::SecurityEvent(event_request) => {
            info!(
                "Processing security event from agent {}: {}",
                agent_id, event_request.event_type
            );

            // Store the security event
            let event = threat_service
                .create_security_event(agent_id, event_request)
                .await
                .map_err(|e| format!("Failed to create security event: {}", e))?;

            // Notify all dashboards about the new event
            let dashboard_message = DashboardMessage::NewSecurityEvent {
                event: event.clone(),
                agent_name: format!("Agent-{}", agent_id), // TODO: Get actual agent name
            };

            connection_manager
                .send_to_all_dashboards(&dashboard_message)
                .await
                .map_err(|e| format!("Failed to notify dashboards: {}", e))?;

            // Check if this event generated any alerts
            let alerts = threat_service
                .get_alerts(Some(agent_id), None)
                .await
                .map_err(|e| format!("Failed to check alerts: {}", e))?;

            // Notify about new alerts if any were created recently
            for alert in alerts.iter().take(1) {
                // Only notify about the most recent alert
                let alert_message = DashboardMessage::NewThreatAlert {
                    alert: alert.clone(),
                    agent_name: format!("Agent-{}", agent_id),
                    event_title: event.title.clone(),
                };

                connection_manager
                    .send_to_all_dashboards(&alert_message)
                    .await
                    .map_err(|e| format!("Failed to notify about alert: {}", e))?;
            }

            Ok(())
        }

        AgentMessage::CommandResponse {
            command_id,
            status,
            result,
        } => {
            info!(
                "Received command response from agent {}: {:?}",
                agent_id, status
            );

            // Update command status in database
            threat_service
                .update_command_status(command_id, status.clone(), result.clone())
                .await
                .map_err(|e| format!("Failed to update command status: {}", e))?;

            // Notify dashboards about command completion
            let command = threat_service
                .get_command_by_id(command_id)
                .await
                .map_err(|e| format!("Failed to get command details: {}", e))?;

            if let Some(command) = command {
                let dashboard_message = DashboardMessage::CommandStatusUpdate { command };
                connection_manager
                    .send_to_all_dashboards(&dashboard_message)
                    .await
                    .map_err(|e| format!("Failed to notify about command status: {}", e))?;
            }

            Ok(())
        }

        AgentMessage::SystemMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_connections,
            running_processes,
        } => {
            debug!(
                "Received system metrics from agent {}: CPU {}%, Memory {}%, Disk {}%",
                agent_id, cpu_usage, memory_usage, disk_usage
            );

            // Store metrics in database
            let metrics_data = serde_json::json!({
                "cpu_usage": cpu_usage,
                "memory_usage": memory_usage,
                "disk_usage": disk_usage,
                "network_connections": network_connections,
                "running_processes": running_processes,
                "timestamp": chrono::Utc::now()
            });

            // TODO: Implement system metrics storage
            // let metrics = threat_service.create_system_metrics(agent_id, "performance", metrics_data).await?;

            // Update agent status
            let heartbeat = secureguard_shared::HeartbeatRequest {
                agent_id,
                status: AgentStatus::Online,
            };

            agent_service
                .update_heartbeat(heartbeat)
                .await
                .map_err(|e| format!("Failed to update agent heartbeat: {}", e))?;

            // Notify dashboards about agent status update
            let dashboard_message = DashboardMessage::AgentStatusUpdate {
                agent_id,
                status: AgentStatus::Online,
                last_seen: chrono::Utc::now(),
            };

            connection_manager
                .send_to_all_dashboards(&dashboard_message)
                .await
                .map_err(|e| format!("Failed to notify about agent status: {}", e))?;

            Ok(())
        }

        _ => {
            warn!("Received unexpected message type from agent {}", agent_id);
            Ok(())
        }
    }
}

// Helper function to send commands to agents
pub async fn send_command_to_agent(
    agent_id: Uuid,
    command_id: Uuid,
    command_type: String,
    command_data: serde_json::Value,
    connection_manager: &ConnectionManager,
) -> Result<(), String> {
    let message = AgentMessage::Command {
        command_id,
        command_type,
        command_data,
    };

    connection_manager.send_to_agent(agent_id, &message).await
}
