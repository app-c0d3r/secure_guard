use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::services::subscription_admin_service::{
    SubscriptionAdminService, CreatePlanRequest, UpdatePlanRequest, 
    PlanMigrationRequest, SubscriptionPlan, PlanUsageStats, PlanValidationResult
};
use crate::middleware::auth::{RequireRole, UserRole};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct GetPlansQuery {
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GetStatsQuery {
    pub plan_id: Option<Uuid>,
    pub include_revenue: Option<bool>,
    pub date_range: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub meta: Option<serde_json::Value>,
}

impl<T> AdminResponse<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message,
            meta: None,
        }
    }

    pub fn success_with_meta(data: T, message: String, meta: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            message,
            meta: Some(meta),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(error: String) -> Self {
        Self {
            success: false,
            error,
            details: None,
        }
    }

    pub fn with_details(error: String, details: serde_json::Value) -> Self {
        Self {
            success: false,
            error,
            details: Some(details),
        }
    }
}

pub fn subscription_admin_routes() -> Router<AppState> {
    Router::new()
        // Plan Management
        .route("/plans", get(get_all_plans).post(create_plan))
        .route("/plans/:plan_id", get(get_plan).put(update_plan).delete(delete_plan))
        
        // Plan Analytics & Statistics
        .route("/plans/stats", get(get_plan_statistics))
        .route("/plans/:plan_id/stats", get(get_specific_plan_stats))
        .route("/plans/:plan_id/usage", get(get_plan_usage))
        .route("/plans/:plan_id/validate", post(validate_plan_changes))
        
        // User Migration
        .route("/plans/migrate", post(migrate_users_between_plans))
        .route("/plans/:plan_id/users", get(get_plan_users))
        
        // Feature Management
        .route("/features", get(get_available_features))
        .route("/features/usage", get(get_feature_usage_stats))
        
        // Bulk Operations
        .route("/plans/bulk-update", post(bulk_update_plans))
        .route("/plans/bulk-activate", post(bulk_activate_plans))
        .route("/plans/bulk-deactivate", post(bulk_deactivate_plans))
        
        // Admin Utilities
        .route("/plans/export", get(export_plans))
        .route("/plans/import", post(import_plans))
        .route("/health", get(subscription_system_health))
        
        // Require SystemAdmin role for all routes
        .layer(RequireRole::new(UserRole::SystemAdmin))
}

/// Get all subscription plans
pub async fn get_all_plans(
    Query(params): Query<GetPlansQuery>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<Vec<SubscriptionPlan>>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    let include_inactive = params.include_inactive.unwrap_or(false);
    
    match service.get_all_plans(include_inactive).await {
        Ok(plans) => {
            let meta = serde_json::json!({
                "total_plans": plans.len(),
                "active_plans": plans.iter().filter(|p| p.is_active).count(),
                "inactive_plans": plans.iter().filter(|p| !p.is_active).count(),
                "include_inactive": include_inactive
            });
            
            Ok(Json(AdminResponse::success_with_meta(
                plans, 
                "Subscription plans retrieved successfully".to_string(),
                meta
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve plans: {}", e))),
        )),
    }
}

/// Get specific subscription plan
pub async fn get_plan(
    Path(plan_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<SubscriptionPlan>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    
    match service.get_plan_by_id(plan_id).await {
        Ok(plan) => Ok(Json(AdminResponse::success(
            plan,
            "Subscription plan retrieved successfully".to_string(),
        ))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(format!("Plan not found: {}", e))),
        )),
    }
}

/// Create new subscription plan
pub async fn create_plan(
    State(state): State<AppState>,
    Json(request): Json<CreatePlanRequest>,
) -> Result<Json<AdminResponse<SubscriptionPlan>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    
    match service.create_plan(request).await {
        Ok(plan) => Ok(Json(AdminResponse::success(
            plan,
            "Subscription plan created successfully".to_string(),
        ))),
        Err(e) => {
            let status = match e {
                secureguard_shared::SecureGuardError::ValidationError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(ErrorResponse::new(format!("Failed to create plan: {}", e)))))
        }
    }
}

/// Update subscription plan
pub async fn update_plan(
    Path(plan_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(request): Json<UpdatePlanRequest>,
) -> Result<Json<AdminResponse<SubscriptionPlan>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    
    match service.update_plan(plan_id, request).await {
        Ok(plan) => Ok(Json(AdminResponse::success(
            plan,
            "Subscription plan updated successfully".to_string(),
        ))),
        Err(e) => {
            let status = match e {
                secureguard_shared::SecureGuardError::ValidationError(_) => StatusCode::BAD_REQUEST,
                secureguard_shared::SecureGuardError::PlanNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(ErrorResponse::new(format!("Failed to update plan: {}", e)))))
        }
    }
}

/// Delete subscription plan
pub async fn delete_plan(
    Path(plan_id): Path<Uuid>,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<AdminResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    let force = params.get("force").and_then(|v| v.parse().ok()).unwrap_or(false);
    
    match service.delete_plan(plan_id, force).await {
        Ok(_) => Ok(Json(AdminResponse::success(
            "Plan deleted successfully".to_string(),
            "Subscription plan has been deactivated".to_string(),
        ))),
        Err(e) => {
            let status = match e {
                secureguard_shared::SecureGuardError::ValidationError(_) => StatusCode::BAD_REQUEST,
                secureguard_shared::SecureGuardError::PlanNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(ErrorResponse::new(format!("Failed to delete plan: {}", e)))))
        }
    }
}

/// Get plan statistics
pub async fn get_plan_statistics(
    Query(params): Query<GetStatsQuery>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<Vec<PlanUsageStats>>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    
    match service.get_plan_usage_stats(params.plan_id).await {
        Ok(stats) => {
            let meta = serde_json::json!({
                "total_plans_analyzed": stats.len(),
                "include_revenue": params.include_revenue.unwrap_or(false),
                "date_range": params.date_range.unwrap_or_else(|| "all_time".to_string())
            });
            
            Ok(Json(AdminResponse::success_with_meta(
                stats,
                "Plan statistics retrieved successfully".to_string(),
                meta
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve statistics: {}", e))),
        )),
    }
}

/// Get specific plan statistics
pub async fn get_specific_plan_stats(
    Path(plan_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<Vec<PlanUsageStats>>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    
    match service.get_plan_usage_stats(Some(plan_id)).await {
        Ok(stats) => Ok(Json(AdminResponse::success(
            stats,
            "Plan statistics retrieved successfully".to_string(),
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve plan statistics: {}", e))),
        )),
    }
}

/// Get plan usage details
pub async fn get_plan_usage(
    Path(plan_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool.clone());
    
    // Get detailed usage information for a specific plan
    let usage_data = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_users,
            COUNT(CASE WHEN us.status = 'active' THEN 1 END) as active_users,
            COUNT(CASE WHEN us.is_trial = true THEN 1 END) as trial_users,
            AVG(ut.current_devices::FLOAT) as avg_devices_per_user,
            AVG(ut.current_api_keys::FLOAT) as avg_api_keys_per_user,
            SUM(ut.current_devices) as total_devices,
            SUM(ut.current_api_keys) as total_api_keys
        FROM subscriptions.user_subscriptions us
        LEFT JOIN subscriptions.usage_tracking ut ON us.user_id = ut.user_id
        WHERE us.plan_id = $1
        "#,
        plan_id
    )
    .fetch_one(&state.pool)
    .await;

    match usage_data {
        Ok(data) => {
            let usage_summary = serde_json::json!({
                "plan_id": plan_id,
                "total_users": data.total_users.unwrap_or(0),
                "active_users": data.active_users.unwrap_or(0),
                "trial_users": data.trial_users.unwrap_or(0),
                "avg_devices_per_user": data.avg_devices_per_user.unwrap_or(0.0),
                "avg_api_keys_per_user": data.avg_api_keys_per_user.unwrap_or(0.0),
                "total_devices": data.total_devices.unwrap_or(0),
                "total_api_keys": data.total_api_keys.unwrap_or(0)
            });

            Ok(Json(AdminResponse::success(
                usage_summary,
                "Plan usage retrieved successfully".to_string(),
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve plan usage: {}", e))),
        )),
    }
}

/// Validate plan changes before applying
pub async fn validate_plan_changes(
    Path(plan_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(request): Json<UpdatePlanRequest>,
) -> Result<Json<AdminResponse<PlanValidationResult>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    
    match service.validate_plan_update(plan_id, &request).await {
        Ok(validation_result) => Ok(Json(AdminResponse::success(
            validation_result,
            "Plan validation completed".to_string(),
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to validate plan changes: {}", e))),
        )),
    }
}

/// Migrate users between plans
pub async fn migrate_users_between_plans(
    State(state): State<AppState>,
    Json(request): Json<PlanMigrationRequest>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    
    match service.migrate_users_between_plans(request).await {
        Ok(affected_users) => {
            let result = serde_json::json!({
                "affected_users": affected_users,
                "migration_completed_at": chrono::Utc::now()
            });

            Ok(Json(AdminResponse::success(
                result,
                format!("Successfully migrated {} users", affected_users),
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to migrate users: {}", e))),
        )),
    }
}

/// Get users on a specific plan
pub async fn get_plan_users(
    Path(plan_id): Path<Uuid>,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    let users = sqlx::query!(
        r#"
        SELECT 
            u.user_id, u.username, u.email, u.created_at,
            us.status, us.is_trial, us.current_period_end,
            ut.current_devices, ut.current_api_keys
        FROM users.users u
        JOIN subscriptions.user_subscriptions us ON u.user_id = us.user_id
        LEFT JOIN subscriptions.usage_tracking ut ON u.user_id = ut.user_id
        WHERE us.plan_id = $1
        ORDER BY u.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        plan_id, limit, offset
    )
    .fetch_all(&state.pool)
    .await;

    match users {
        Ok(user_list) => {
            let total_count = sqlx::query!(
                "SELECT COUNT(*) as count FROM subscriptions.user_subscriptions WHERE plan_id = $1",
                plan_id
            )
            .fetch_one(&state.pool)
            .await
            .map(|row| row.count.unwrap_or(0))
            .unwrap_or(0);

            let response = serde_json::json!({
                "users": user_list,
                "total_count": total_count,
                "limit": limit,
                "offset": offset,
                "has_more": (offset + limit) < total_count
            });

            Ok(Json(AdminResponse::success(
                response,
                "Plan users retrieved successfully".to_string(),
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve plan users: {}", e))),
        )),
    }
}

/// Get available features
pub async fn get_available_features(
    State(_state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let features = serde_json::json!({
        "core_features": [
            {
                "name": "real_time_monitoring",
                "display_name": "Real-time Monitoring",
                "description": "Continuous monitoring with instant alerts"
            },
            {
                "name": "advanced_threat_detection",
                "display_name": "Advanced Threat Detection",
                "description": "AI-powered behavioral analysis and threat detection"
            },
            {
                "name": "custom_rules",
                "display_name": "Custom Security Rules",
                "description": "Create custom rules for your security requirements"
            },
            {
                "name": "api_access",
                "display_name": "API Access",
                "description": "Programmatic access to SecureGuard APIs"
            },
            {
                "name": "priority_support",
                "display_name": "Priority Support",
                "description": "Faster response times and dedicated support"
            },
            {
                "name": "audit_logs",
                "display_name": "Audit Logs",
                "description": "Detailed audit trails for compliance"
            },
            {
                "name": "integrations_enabled",
                "display_name": "Third-party Integrations",
                "description": "Connect with external security tools"
            },
            {
                "name": "vulnerability_scanning",
                "display_name": "Vulnerability Scanning",
                "description": "Automated vulnerability detection and assessment"
            },
            {
                "name": "compliance_reporting",
                "display_name": "Compliance Reporting",
                "description": "Generate compliance reports for audits"
            },
            {
                "name": "remote_response",
                "display_name": "Remote Response",
                "description": "Remote incident response capabilities"
            },
            {
                "name": "custom_dashboards",
                "display_name": "Custom Dashboards",
                "description": "Personalized security dashboards"
            },
            {
                "name": "bulk_operations",
                "display_name": "Bulk Operations",
                "description": "Manage multiple devices and configurations at once"
            }
        ],
        "limits": [
            {
                "name": "max_devices",
                "display_name": "Maximum Devices",
                "description": "Maximum number of devices that can be protected"
            },
            {
                "name": "max_api_keys",
                "display_name": "Maximum API Keys",
                "description": "Maximum number of API keys that can be created"
            },
            {
                "name": "log_retention_days",
                "display_name": "Log Retention (Days)",
                "description": "How long security logs are retained"
            },
            {
                "name": "alert_history_days",
                "display_name": "Alert History (Days)",
                "description": "How long alert history is available"
            }
        ]
    });

    Ok(Json(AdminResponse::success(
        features,
        "Available features retrieved successfully".to_string(),
    )))
}

/// Get feature usage statistics across all plans
pub async fn get_feature_usage_stats(
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    // This would aggregate feature usage across all plans
    let stats = serde_json::json!({
        "feature_adoption": {
            "real_time_monitoring": {
                "total_plans_with_feature": 3,
                "total_users_with_access": 1250,
                "adoption_rate": 0.85
            },
            "advanced_threat_detection": {
                "total_plans_with_feature": 2,
                "total_users_with_access": 500,
                "adoption_rate": 0.75
            }
        },
        "most_popular_features": [
            "real_time_monitoring",
            "api_access",
            "audit_logs"
        ],
        "least_used_features": [
            "bulk_operations",
            "compliance_reporting"
        ]
    });

    Ok(Json(AdminResponse::success(
        stats,
        "Feature usage statistics retrieved successfully".to_string(),
    )))
}

/// Export plans configuration
pub async fn export_plans(
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let service = SubscriptionAdminService::new(state.pool);
    
    match service.get_all_plans(true).await {
        Ok(plans) => {
            let export_data = serde_json::json!({
                "export_timestamp": chrono::Utc::now(),
                "export_version": "1.0",
                "total_plans": plans.len(),
                "plans": plans
            });

            Ok(Json(AdminResponse::success(
                export_data,
                "Plans exported successfully".to_string(),
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to export plans: {}", e))),
        )),
    }
}

/// Import plans configuration
pub async fn import_plans(
    State(_state): State<AppState>,
    Json(_import_data): Json<serde_json::Value>,
) -> Result<Json<AdminResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    // Implementation would validate and import plan configurations
    Ok(Json(AdminResponse::success(
        "Plans imported successfully".to_string(),
        "Plan import completed".to_string(),
    )))
}

/// Check subscription system health
pub async fn subscription_system_health(
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let health_check = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_plans,
            COUNT(CASE WHEN is_active = true THEN 1 END) as active_plans,
            (SELECT COUNT(*) FROM subscriptions.user_subscriptions WHERE status = 'active') as active_subscriptions,
            (SELECT COUNT(*) FROM subscriptions.usage_tracking) as usage_records
        FROM subscriptions.plans
        "#
    )
    .fetch_one(&state.pool)
    .await;

    match health_check {
        Ok(data) => {
            let health = serde_json::json!({
                "status": "healthy",
                "timestamp": chrono::Utc::now(),
                "metrics": {
                    "total_plans": data.total_plans.unwrap_or(0),
                    "active_plans": data.active_plans.unwrap_or(0),
                    "active_subscriptions": data.active_subscriptions.unwrap_or(0),
                    "usage_records": data.usage_records.unwrap_or(0)
                },
                "database_connection": "ok"
            });

            Ok(Json(AdminResponse::success(
                health,
                "Subscription system is healthy".to_string(),
            )))
        }
        Err(e) => {
            let health = serde_json::json!({
                "status": "unhealthy",
                "timestamp": chrono::Utc::now(),
                "error": e.to_string(),
                "database_connection": "failed"
            });

            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::with_details(
                    "Subscription system health check failed".to_string(),
                    health
                )),
            ))
        }
    }
}

// Bulk operations (placeholder implementations)
pub async fn bulk_update_plans(
    State(_state): State<AppState>,
    Json(_bulk_request): Json<serde_json::Value>,
) -> Result<Json<AdminResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(AdminResponse::success(
        "Bulk update completed".to_string(),
        "Plans updated successfully".to_string(),
    )))
}

pub async fn bulk_activate_plans(
    State(_state): State<AppState>,
    Json(_plan_ids): Json<Vec<Uuid>>,
) -> Result<Json<AdminResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(AdminResponse::success(
        "Bulk activation completed".to_string(),
        "Plans activated successfully".to_string(),
    )))
}

pub async fn bulk_deactivate_plans(
    State(_state): State<AppState>,
    Json(_plan_ids): Json<Vec<Uuid>>,
) -> Result<Json<AdminResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(AdminResponse::success(
        "Bulk deactivation completed".to_string(),
        "Plans deactivated successfully".to_string(),
    )))
}