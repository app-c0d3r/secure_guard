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

use crate::services::role_management_service::{
    RoleManagementService, AssignRoleRequest, UpdateRolePermissionsRequest,
    Role, Permission, UserRoleInfo, RolePermissionAudit
};
use crate::middleware::rbac::{RequirePermission, permissions, UserRole};
use crate::middleware::auth::AuthUser;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct GetUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub filter: Option<String>, // secrets, users, system, all
}

#[derive(Debug, Deserialize)]
pub struct GetAuditQuery {
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
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
}

pub fn role_admin_routes() -> Router<AppState> {
    Router::new()
        // Role Management
        .route("/roles", get(get_all_roles))
        .route("/roles/:role_id/permissions", get(get_role_permissions).put(update_role_permissions))
        
        // Permission Management
        .route("/permissions", get(get_all_permissions))
        .route("/permissions/categories", get(get_permission_categories))
        
        // User Role Management
        .route("/users", get(get_users_with_roles))
        .route("/users/:user_id/roles", get(get_user_roles).post(assign_user_role))
        .route("/users/:user_id/roles/:role_id", delete(remove_user_role))
        .route("/users/:user_id/primary-role", put(set_user_primary_role))
        .route("/users/:user_id/permissions", get(get_user_permissions))
        
        // Sensitive Access Management
        .route("/users/sensitive-access", get(get_users_with_sensitive_access))
        .route("/users/:user_id/secret-access", get(check_user_secret_access))
        
        // Audit and Monitoring
        .route("/audit", get(get_role_audit_trail))
        .route("/audit/users/:user_id", get(get_user_audit_trail))
        
        // System Health
        .route("/health", get(role_system_health))
        
        // Require appropriate permissions for all routes
        .layer(RequirePermission::new(permissions::USERS_ROLES))
}

/// Get all roles in the system
pub async fn get_all_roles(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<Vec<Role>>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    let include_inactive = params.get("include_inactive")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    
    match service.get_all_roles(include_inactive).await {
        Ok(roles) => {
            let meta = serde_json::json!({
                "total_roles": roles.len(),
                "active_roles": roles.iter().filter(|r| r.is_active).count(),
                "system_roles": roles.iter().filter(|r| r.is_system_role).count(),
                "assignable_roles": roles.iter().filter(|r| r.is_assignable).count()
            });
            
            Ok(Json(AdminResponse::success_with_meta(
                roles,
                "Roles retrieved successfully".to_string(),
                meta
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve roles: {}", e))),
        )),
    }
}

/// Get all permissions organized by category
pub async fn get_all_permissions(
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<HashMap<String, Vec<Permission>>>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    
    match service.get_all_permissions().await {
        Ok(permissions) => {
            let total_permissions: usize = permissions.values().map(|v| v.len()).sum();
            let meta = serde_json::json!({
                "total_permissions": total_permissions,
                "categories": permissions.keys().collect::<Vec<_>>(),
                "category_count": permissions.len()
            });
            
            Ok(Json(AdminResponse::success_with_meta(
                permissions,
                "Permissions retrieved successfully".to_string(),
                meta
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve permissions: {}", e))),
        )),
    }
}

/// Get permission categories summary
pub async fn get_permission_categories(
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let categories = serde_json::json!({
        "system": {
            "name": "System Administration",
            "description": "Core system administration and configuration",
            "sensitivity": "critical",
            "permissions": ["system.admin", "system.config", "system.maintenance"]
        },
        "security": {
            "name": "Security & Secrets",
            "description": "Access to sensitive security information and secrets",
            "sensitivity": "critical",
            "permissions": ["secrets.read", "secrets.create", "secrets.update", "secrets.delete", "security.incidents"]
        },
        "users": {
            "name": "User Management", 
            "description": "User account and role management",
            "sensitivity": "high",
            "permissions": ["users.create", "users.read", "users.update", "users.delete", "users.roles"]
        },
        "agents": {
            "name": "Agent Management",
            "description": "Device and agent management",
            "sensitivity": "medium",
            "permissions": ["agents.read", "agents.create", "agents.update", "agents.delete", "agents.control"]
        },
        "subscriptions": {
            "name": "Subscription Management",
            "description": "Subscription plans and billing management",
            "sensitivity": "high",
            "permissions": ["subscriptions.read", "subscriptions.create", "subscriptions.update", "subscriptions.migrate"]
        },
        "audit": {
            "name": "Audit & Compliance",
            "description": "Audit logs and compliance reporting",
            "sensitivity": "medium",
            "permissions": ["audit.read", "audit.export"]
        },
        "api": {
            "name": "API Access",
            "description": "Programmatic API access levels",
            "sensitivity": "medium",
            "permissions": ["api.read", "api.write", "api.admin"]
        }
    });
    
    Ok(Json(AdminResponse::success(
        categories,
        "Permission categories retrieved successfully".to_string()
    )))
}

/// Get users with their roles and permissions summary
pub async fn get_users_with_roles(
    Query(params): Query<GetUsersQuery>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<Vec<UserRoleInfo>>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    
    match service.get_user_roles_summary(params.limit, params.offset).await {
        Ok(mut users) => {
            // Apply filter if specified
            if let Some(filter) = &params.filter {
                users = match filter.as_str() {
                    "secrets" => users.into_iter().filter(|u| u.can_access_secrets).collect(),
                    "users" => users.into_iter().filter(|u| u.can_manage_users).collect(),
                    "system" => users.into_iter().filter(|u| u.can_admin_system).collect(),
                    _ => users,
                };
            }
            
            let meta = serde_json::json!({
                "total_users": users.len(),
                "users_with_secrets_access": users.iter().filter(|u| u.can_access_secrets).count(),
                "users_with_user_management": users.iter().filter(|u| u.can_manage_users).count(),
                "users_with_system_admin": users.iter().filter(|u| u.can_admin_system).count(),
                "filter_applied": params.filter
            });
            
            Ok(Json(AdminResponse::success_with_meta(
                users,
                "Users with roles retrieved successfully".to_string(),
                meta
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve users: {}", e))),
        )),
    }
}

/// Get specific user's roles and permissions
pub async fn get_user_roles(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<UserRoleInfo>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    
    match service.get_user_role_details(user_id).await {
        Ok(user_info) => Ok(Json(AdminResponse::success(
            user_info,
            "User roles retrieved successfully".to_string(),
        ))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(format!("User not found: {}", e))),
        )),
    }
}

/// Assign role to user
pub async fn assign_user_role(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    AuthUser(admin_user): AuthUser,
    Json(mut request): Json<AssignRoleRequest>,
) -> Result<Json<AdminResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    
    // Override user_id from path
    request.user_id = user_id;
    
    match service.assign_role_to_user(request, admin_user.user_id, None).await {
        Ok(user_role_id) => Ok(Json(AdminResponse::success(
            format!("Role assigned successfully: {}", user_role_id),
            "Role assigned to user".to_string(),
        ))),
        Err(e) => {
            let status = match e {
                secureguard_shared::SecureGuardError::ValidationError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(ErrorResponse::new(format!("Failed to assign role: {}", e)))))
        }
    }
}

/// Remove role from user
pub async fn remove_user_role(
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    AuthUser(admin_user): AuthUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<AdminResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    let reason = params.get("reason").cloned();
    
    match service.remove_role_from_user(user_id, role_id, admin_user.user_id, reason, None).await {
        Ok(_) => Ok(Json(AdminResponse::success(
            "Role removed successfully".to_string(),
            "Role removed from user".to_string(),
        ))),
        Err(e) => {
            let status = match e {
                secureguard_shared::SecureGuardError::ValidationError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(ErrorResponse::new(format!("Failed to remove role: {}", e)))))
        }
    }
}

/// Set user's primary role
pub async fn set_user_primary_role(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    AuthUser(admin_user): AuthUser,
    Json(role_request): Json<serde_json::Value>,
) -> Result<Json<AdminResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    
    let role_id = role_request.get("role_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("Invalid role_id".to_string())))
        })?;
    
    match service.set_user_primary_role(user_id, role_id, admin_user.user_id).await {
        Ok(_) => Ok(Json(AdminResponse::success(
            "Primary role set successfully".to_string(),
            "User primary role updated".to_string(),
        ))),
        Err(e) => {
            let status = match e {
                secureguard_shared::SecureGuardError::ValidationError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(ErrorResponse::new(format!("Failed to set primary role: {}", e)))))
        }
    }
}

/// Get user's effective permissions
pub async fn get_user_permissions(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    // Get user permissions by category
    let permissions_by_category = sqlx::query!(
        r#"
        SELECT category, 
               array_agg(permission_slug ORDER BY permission_name) as permissions,
               array_agg(permission_name ORDER BY permission_name) as permission_names,
               array_agg(sensitivity_level ORDER BY permission_name) as sensitivity_levels
        FROM rbac.user_effective_permissions
        WHERE user_id = $1
        GROUP BY category
        ORDER BY category
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await;

    match permissions_by_category {
        Ok(data) => {
            let mut permissions_map = serde_json::Map::new();
            let mut total_permissions = 0;
            let mut high_sensitivity_count = 0;
            
            for row in data {
                let permissions = row.permissions.unwrap_or_default();
                let permission_names = row.permission_names.unwrap_or_default();
                let sensitivity_levels = row.sensitivity_levels.unwrap_or_default();
                
                total_permissions += permissions.len();
                high_sensitivity_count += sensitivity_levels.iter().filter(|&&level| level >= 3).count();
                
                let category_data = serde_json::json!({
                    "permissions": permissions,
                    "permission_names": permission_names,
                    "sensitivity_levels": sensitivity_levels,
                    "count": permissions.len()
                });
                
                permissions_map.insert(row.category, category_data);
            }
            
            let result = serde_json::json!({
                "user_id": user_id,
                "permissions_by_category": permissions_map,
                "summary": {
                    "total_permissions": total_permissions,
                    "high_sensitivity_permissions": high_sensitivity_count,
                    "categories": permissions_map.keys().collect::<Vec<_>>()
                }
            });
            
            Ok(Json(AdminResponse::success(
                result,
                "User permissions retrieved successfully".to_string(),
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve user permissions: {}", e))),
        )),
    }
}

/// Get users with sensitive access
pub async fn get_users_with_sensitive_access(
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<Vec<UserRoleInfo>>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    
    match service.get_users_with_sensitive_access().await {
        Ok(users) => {
            let meta = serde_json::json!({
                "total_sensitive_users": users.len(),
                "users_with_secrets": users.iter().filter(|u| u.can_access_secrets).count(),
                "users_with_user_mgmt": users.iter().filter(|u| u.can_manage_users).count(),
                "users_with_system_admin": users.iter().filter(|u| u.can_admin_system).count()
            });
            
            Ok(Json(AdminResponse::success_with_meta(
                users,
                "Users with sensitive access retrieved successfully".to_string(),
                meta
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve sensitive users: {}", e))),
        )),
    }
}

/// Check if user can access secrets
pub async fn check_user_secret_access(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    
    match service.user_can_access_secrets(user_id).await {
        Ok(can_access) => {
            let result = serde_json::json!({
                "user_id": user_id,
                "can_access_secrets": can_access,
                "checked_at": chrono::Utc::now()
            });
            
            Ok(Json(AdminResponse::success(
                result,
                format!("User {} access secrets: {}", if can_access { "can" } else { "cannot" }, can_access),
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to check secret access: {}", e))),
        )),
    }
}

/// Get role audit trail
pub async fn get_role_audit_trail(
    Query(params): Query<GetAuditQuery>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<Vec<RolePermissionAudit>>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    
    match service.get_role_audit_trail(params.user_id, params.limit).await {
        Ok(audit_trail) => {
            let meta = serde_json::json!({
                "total_entries": audit_trail.len(),
                "user_filter": params.user_id,
                "actions": audit_trail.iter().map(|a| &a.action).collect::<std::collections::HashSet<_>>()
            });
            
            Ok(Json(AdminResponse::success_with_meta(
                audit_trail,
                "Audit trail retrieved successfully".to_string(),
                meta
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve audit trail: {}", e))),
        )),
    }
}

/// Get audit trail for specific user
pub async fn get_user_audit_trail(
    Path(user_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<Vec<RolePermissionAudit>>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    let limit = params.get("limit").and_then(|v| v.parse().ok());
    
    match service.get_role_audit_trail(Some(user_id), limit).await {
        Ok(audit_trail) => Ok(Json(AdminResponse::success(
            audit_trail,
            "User audit trail retrieved successfully".to_string(),
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve user audit trail: {}", e))),
        )),
    }
}

/// Update role permissions
pub async fn update_role_permissions(
    Path(role_id): Path<Uuid>,
    State(state): State<AppState>,
    AuthUser(admin_user): AuthUser,
    Json(mut request): Json<UpdateRolePermissionsRequest>,
) -> Result<Json<AdminResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    let service = RoleManagementService::new(state.pool);
    
    // Override role_id from path
    request.role_id = role_id;
    
    match service.update_role_permissions(request, admin_user.user_id, None).await {
        Ok(_) => Ok(Json(AdminResponse::success(
            "Role permissions updated successfully".to_string(),
            "Permissions have been updated for the role".to_string(),
        ))),
        Err(e) => {
            let status = match e {
                secureguard_shared::SecureGuardError::ValidationError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(ErrorResponse::new(format!("Failed to update role permissions: {}", e)))))
        }
    }
}

/// Get role permissions
pub async fn get_role_permissions(
    Path(role_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let permissions = sqlx::query!(
        r#"
        SELECT p.permission_id, p.permission_slug, p.permission_name, 
               p.display_name, p.category, p.action, p.sensitivity_level,
               rp.granted_at, rp.granted_by
        FROM rbac.role_permissions rp
        JOIN rbac.permissions p ON rp.permission_id = p.permission_id
        WHERE rp.role_id = $1 AND p.is_active = TRUE
        ORDER BY p.category, p.permission_name
        "#,
        role_id
    )
    .fetch_all(&state.pool)
    .await;

    match permissions {
        Ok(perms) => {
            let result = serde_json::json!({
                "role_id": role_id,
                "permissions": perms,
                "total_permissions": perms.len(),
                "categories": perms.iter().map(|p| &p.category).collect::<std::collections::HashSet<_>>()
            });
            
            Ok(Json(AdminResponse::success(
                result,
                "Role permissions retrieved successfully".to_string(),
            )))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to retrieve role permissions: {}", e))),
        )),
    }
}

/// Check role system health
pub async fn role_system_health(
    State(state): State<AppState>,
) -> Result<Json<AdminResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let health_data = sqlx::query!(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM rbac.roles WHERE is_active = TRUE) as active_roles,
            (SELECT COUNT(*) FROM rbac.permissions WHERE is_active = TRUE) as active_permissions,
            (SELECT COUNT(*) FROM rbac.user_roles WHERE is_active = TRUE) as active_user_roles,
            (SELECT COUNT(*) FROM rbac.role_permissions) as role_permission_mappings,
            (SELECT COUNT(DISTINCT user_id) FROM rbac.user_roles WHERE is_active = TRUE) as users_with_roles
        "#
    )
    .fetch_one(&state.pool)
    .await;

    match health_data {
        Ok(data) => {
            let health = serde_json::json!({
                "status": "healthy",
                "timestamp": chrono::Utc::now(),
                "metrics": {
                    "active_roles": data.active_roles.unwrap_or(0),
                    "active_permissions": data.active_permissions.unwrap_or(0),
                    "active_user_roles": data.active_user_roles.unwrap_or(0),
                    "role_permission_mappings": data.role_permission_mappings.unwrap_or(0),
                    "users_with_roles": data.users_with_roles.unwrap_or(0)
                },
                "database_connection": "ok"
            });

            Ok(Json(AdminResponse::success(
                health,
                "Role system is healthy".to_string(),
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
                Json(ErrorResponse::new("Role system health check failed".to_string())),
            ))
        }
    }
}