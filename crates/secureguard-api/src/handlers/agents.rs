use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use secureguard_shared::{RegisterAgentRequest, HeartbeatRequest, Agent, SecureGuardError};
use crate::{
    database::Database,
    services::agent_service::AgentService,
    middleware::auth::AuthUser,
};

pub async fn register_agent(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
    Json(request): Json<RegisterAgentRequest>,
) -> Result<(StatusCode, Json<Agent>), (StatusCode, Json<serde_json::Value>)> {
    let agent_service = AgentService::new(db.pool().clone());
    
    let default_tenant_id = Uuid::new_v4();
    
    let agent = agent_service.register_agent(default_tenant_id, request).await
        .map_err(|e| handle_error(e))?;

    Ok((StatusCode::CREATED, Json(agent)))
}

pub async fn heartbeat(
    State(db): State<Database>,
    Json(request): Json<HeartbeatRequest>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let agent_service = AgentService::new(db.pool().clone());
    
    agent_service.update_heartbeat(request).await
        .map_err(|e| handle_error(e))?;

    Ok(StatusCode::OK)
}

pub async fn list_agents(
    State(db): State<Database>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<Agent>>, (StatusCode, Json<serde_json::Value>)> {
    let agent_service = AgentService::new(db.pool().clone());
    
    let default_tenant_id = Uuid::new_v4();
    
    let agents = agent_service.list_agents_for_tenant(default_tenant_id).await
        .map_err(|e| handle_error(e))?;

    Ok(Json(agents))
}

fn handle_error(error: SecureGuardError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, message) = match error {
        SecureGuardError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        SecureGuardError::AgentNotFound => (StatusCode::NOT_FOUND, "Agent not found".to_string()),
        SecureGuardError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
    };

    (status, Json(serde_json::json!({ "error": message })))
}