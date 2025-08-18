use axum::{
    extract::{State, Query},
    response::Response,
    http::StatusCode,
};
use axum::extract::ws::WebSocketUpgrade;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::Database,
    websocket::{
        connection_manager::ConnectionManager,
        agent_handler::{handle_agent_websocket, AgentWebSocketQuery},
        dashboard_handler::{handle_dashboard_websocket, DashboardWebSocketQuery},
    },
};

// Re-export WebSocket handlers for use in routes
pub async fn agent_websocket(
    ws: WebSocketUpgrade,
    Query(params): Query<AgentWebSocketQuery>,
    State(db): State<Database>,
    State(connection_manager): State<ConnectionManager>,
) -> Result<Response, StatusCode> {
    handle_agent_websocket(ws, Query(params), State(db), State(connection_manager)).await
}

pub async fn dashboard_websocket(
    ws: WebSocketUpgrade,
    Query(params): Query<DashboardWebSocketQuery>,
    State(db): State<Database>,
    State(connection_manager): State<ConnectionManager>,
) -> Result<Response, StatusCode> {
    handle_dashboard_websocket(ws, Query(params), State(db), State(connection_manager)).await
}

// Health check endpoint for WebSocket infrastructure
pub async fn websocket_health(
    State(connection_manager): State<ConnectionManager>,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
    let (agent_count, dashboard_count, connected_agents) = {
        let (agents, dashboards) = connection_manager.get_connection_count().await;
        let agents_list = connection_manager.get_connected_agents().await;
        (agents, dashboards, agents_list)
    };

    Ok(axum::Json(serde_json::json!({
        "status": "healthy",
        "websocket_connections": {
            "agents": agent_count,
            "dashboards": dashboard_count,
            "total": agent_count + dashboard_count
        },
        "connected_agents": connected_agents.len(),
        "timestamp": chrono::Utc::now()
    })))
}

// WebSocket connection statistics endpoint
pub async fn websocket_stats(
    State(connection_manager): State<ConnectionManager>,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
    let (agent_count, dashboard_count, connected_agents) = {
        let (agents, dashboards) = connection_manager.get_connection_count().await;
        let agents_list = connection_manager.get_connected_agents().await;
        (agents, dashboards, agents_list)
    };

    Ok(axum::Json(serde_json::json!({
        "connections": {
            "agents": {
                "count": agent_count,
                "connected_agent_ids": connected_agents
            },
            "dashboards": {
                "count": dashboard_count
            },
            "total": agent_count + dashboard_count
        },
        "uptime": "N/A", // TODO: Track WebSocket server uptime
        "message_stats": {
            "total_sent": "N/A", // TODO: Implement message counters
            "total_received": "N/A"
        },
        "timestamp": chrono::Utc::now()
    })))
}