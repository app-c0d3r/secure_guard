pub mod agent_communication;
pub mod agent_service;
pub mod api_key_service;
pub mod auth_service;
pub mod event_processor;
pub mod processing_pipeline;
pub mod realtime_service;
pub mod subscription_service;
pub mod threat_service;
pub mod user_service;
// pub mod config_service;  // Temporarily disabled due to compilation errors
// pub mod remote_command_service;  // Temporarily disabled
// pub mod security_monitoring_service;  // Temporarily disabled due to bigdecimal dependency
// pub mod notification_service;  // Temporarily disabled due to missing columns
// pub mod subscription_admin_service;  // Temporarily disabled
// pub mod role_management_service;  // Temporarily disabled due to missing RBAC tables

#[cfg(test)]
pub mod test_utils;
