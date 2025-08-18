use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{
    database::Database,
    handlers::{auth, agents, threats, pipeline, agent_communication},
};

pub fn create_api_routes() -> Router<Database> {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/agents", agent_routes())
        .nest("/threats", threat_routes())
        .nest("/pipeline", pipeline_routes())
        .nest("/agent-comm", agent_communication_routes())
}

fn auth_routes() -> Router<Database> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/me", get(auth::me))
}

fn agent_routes() -> Router<Database> {
    Router::new()
        .route("/register", post(agents::register_agent))
        .route("/heartbeat", post(agents::heartbeat))
        .route("/", get(agents::list_agents))
}

fn threat_routes() -> Router<Database> {
    Router::new()
        // Security Events
        .route("/events", get(threats::get_security_events))
        .route("/events/bulk", post(threats::bulk_create_events))
        .route("/agents/:agent_id/events", post(threats::create_security_event))
        
        // Detection Rules
        .route("/rules", get(threats::get_detection_rules))
        .route("/rules", post(threats::create_detection_rule))
        
        // Threat Alerts
        .route("/alerts", get(threats::get_alerts))
        .route("/alerts", post(threats::create_alert))
        .route("/alerts/:alert_id", put(threats::update_alert))
        
        // Agent Commands
        .route("/agents/:agent_id/commands", post(threats::create_command))
        .route("/commands/:command_id", put(threats::update_command_status))
        .route("/commands/:command_id", get(threats::get_command))
        
        // Analytics & Intelligence
        .route("/summary", get(threats::get_threat_summary))
        .route("/agents/:agent_id/analysis", get(threats::analyze_threat_patterns))
        .route("/timeline", get(threats::get_threat_timeline))
        .route("/top", get(threats::get_top_threats))
}

fn pipeline_routes() -> Router<Database> {
    Router::new()
        // Pipeline Status and Health
        .route("/status", get(pipeline::get_pipeline_status))
        .route("/metrics", get(pipeline::get_processing_metrics))
        .route("/health", get(pipeline::get_pipeline_status))
        
        // Event Processing
        .route("/events/batch", post(pipeline::process_events_batch))
        .route("/history", get(pipeline::get_processing_history))
        
        // Emergency Controls
        .route("/emergency/stop", post(pipeline::trigger_emergency_stop))
        .route("/emergency/isolate", post(pipeline::emergency_isolate_agents))
        
        // Optimization and Maintenance
        .route("/optimize", post(pipeline::trigger_pipeline_optimization))
        .route("/maintenance", post(pipeline::trigger_system_maintenance))
        
        // Analytics and Intelligence
        .route("/analytics/performance", get(pipeline::get_performance_analytics))
        .route("/analytics/threats", get(pipeline::get_threat_intelligence_summary))
}

fn agent_communication_routes() -> Router<Database> {
    Router::new()
        // Agent Registration and WebSocket Communication
        .route("/register", get(agent_communication::register_agent_enhanced))
        
        // Agent Status and Monitoring
        .route("/status", get(agent_communication::list_agents_with_status))
        .route("/status/:agent_id", get(agent_communication::get_agent_status_detailed))
        .route("/overview", get(agent_communication::get_communication_overview))
        
        // Command Distribution
        .route("/command/:agent_id", post(agent_communication::send_command_to_agent))
        .route("/command/bulk", post(agent_communication::send_bulk_command))
        .route("/command/broadcast", post(agent_communication::broadcast_emergency_command))
        
        // Emergency Operations
        .route("/emergency/isolate", post(agent_communication::emergency_isolate_agents))
        
        // Performance and Analytics
        .route("/metrics/:agent_id", get(agent_communication::get_agent_performance_metrics))
}