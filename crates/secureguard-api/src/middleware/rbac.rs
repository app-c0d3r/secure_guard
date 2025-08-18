use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashSet;
use tracing::{warn, debug};

use crate::middleware::auth::AuthUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    SystemAdmin,
    SecurityAnalyst,
    Admin,
    Manager,
    PowerUser,
    User,
    ReadOnly,
    Guest,
}

impl UserRole {
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "system_admin" => Some(UserRole::SystemAdmin),
            "security_analyst" => Some(UserRole::SecurityAnalyst),
            "admin" => Some(UserRole::Admin),
            "manager" => Some(UserRole::Manager),
            "power_user" => Some(UserRole::PowerUser),
            "user" => Some(UserRole::User),
            "read_only" => Some(UserRole::ReadOnly),
            "guest" => Some(UserRole::Guest),
            _ => None,
        }
    }

    pub fn to_slug(&self) -> &'static str {
        match self {
            UserRole::SystemAdmin => "system_admin",
            UserRole::SecurityAnalyst => "security_analyst",
            UserRole::Admin => "admin",
            UserRole::Manager => "manager",
            UserRole::PowerUser => "power_user",
            UserRole::User => "user",
            UserRole::ReadOnly => "read_only",
            UserRole::Guest => "guest",
        }
    }

    pub fn hierarchy_level(&self) -> u8 {
        match self {
            UserRole::SystemAdmin => 100,
            UserRole::SecurityAnalyst => 80,
            UserRole::Admin => 70,
            UserRole::Manager => 50,
            UserRole::PowerUser => 30,
            UserRole::User => 10,
            UserRole::ReadOnly => 5,
            UserRole::Guest => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserPermissions {
    pub user_id: Uuid,
    pub roles: Vec<UserRole>,
    pub permissions: HashSet<String>,
    pub highest_role: UserRole,
    pub can_access_secrets: bool,
    pub can_manage_users: bool,
    pub can_admin_system: bool,
}

impl UserPermissions {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        permissions.iter().any(|p| self.permissions.contains(*p))
    }

    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|p| self.permissions.contains(*p))
    }

    pub fn has_role(&self, role: &UserRole) -> bool {
        self.roles.iter().any(|r| r.to_slug() == role.to_slug())
    }

    pub fn has_role_level_or_higher(&self, min_role: &UserRole) -> bool {
        self.highest_role.hierarchy_level() >= min_role.hierarchy_level()
    }

    pub fn can_access_resource(&self, resource_type: &str, action: &str) -> bool {
        let permission = format!("{}.{}", resource_type, action);
        self.has_permission(&permission)
    }

    pub fn can_manage_resource(&self, resource_type: &str) -> bool {
        self.has_any_permission(&[
            &format!("{}.create", resource_type),
            &format!("{}.update", resource_type),
            &format!("{}.delete", resource_type),
            &format!("{}.admin", resource_type),
        ])
    }
}

pub struct RequirePermission {
    permission: String,
}

impl RequirePermission {
    pub fn new(permission: impl Into<String>) -> Self {
        Self {
            permission: permission.into(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequirePermission
where
    S: Send + Sync + Clone + 'static,
    PgPool: FromRequestParts<S>,
    AuthUser: FromRequestParts<S>,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        let pool = PgPool::from_request_parts(parts, state).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Database connection failed" })),
            )
        })?;

        // Get required permission from request extensions (set by middleware)
        let required_permission = parts
            .extensions
            .get::<String>()
            .ok_or_else(|| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Permission requirement not set" })),
                )
            })?;

        let user_permissions = get_user_permissions(&pool, auth_user.0.user_id).await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Failed to check permissions" })),
                )
            })?;

        if !user_permissions.has_permission(required_permission) {
            warn!(
                "User {} denied access - missing permission: {}",
                auth_user.0.user_id, required_permission
            );
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "Insufficient permissions",
                    "required_permission": required_permission
                })),
            ));
        }

        Ok(RequirePermission {
            permission: required_permission.clone(),
        })
    }
}

pub struct RequireRole {
    role: UserRole,
}

impl RequireRole {
    pub fn new(role: UserRole) -> Self {
        Self { role }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireRole
where
    S: Send + Sync + Clone + 'static,
    PgPool: FromRequestParts<S>,
    AuthUser: FromRequestParts<S>,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        let pool = PgPool::from_request_parts(parts, state).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Database connection failed" })),
            )
        })?;

        let user_permissions = get_user_permissions(&pool, auth_user.0.user_id).await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Failed to check permissions" })),
                )
            })?;

        // Get required role from request extensions (set by middleware)
        let required_role = parts
            .extensions
            .get::<UserRole>()
            .cloned()
            .unwrap_or_else(|| UserRole::User);

        if !user_permissions.has_role_level_or_higher(&required_role) {
            warn!(
                "User {} denied access - insufficient role level. Has: {:?}, Required: {:?}",
                auth_user.0.user_id, user_permissions.highest_role, required_role
            );
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "Insufficient role level",
                    "current_role": user_permissions.highest_role.to_slug(),
                    "required_role": required_role.to_slug()
                })),
            ));
        }

        Ok(RequireRole { role: required_role })
    }
}

pub struct RequireSecretAccess;

#[async_trait]
impl<S> FromRequestParts<S> for RequireSecretAccess
where
    S: Send + Sync + Clone + 'static,
    PgPool: FromRequestParts<S>,
    AuthUser: FromRequestParts<S>,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        let pool = PgPool::from_request_parts(parts, state).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Database connection failed" })),
            )
        })?;

        let user_permissions = get_user_permissions(&pool, auth_user.0.user_id).await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Failed to check permissions" })),
                )
            })?;

        if !user_permissions.can_access_secrets {
            warn!(
                "User {} denied access to secrets - insufficient permissions",
                auth_user.0.user_id
            );
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "Access to secrets denied",
                    "message": "You do not have permission to access sensitive information"
                })),
            ));
        }

        Ok(RequireSecretAccess)
    }
}

/// Get user permissions from database
pub async fn get_user_permissions(pool: &PgPool, user_id: Uuid) -> Result<UserPermissions, sqlx::Error> {
    // Get user's roles
    let roles_data = sqlx::query!(
        r#"
        SELECT r.role_slug, r.hierarchy_level
        FROM rbac.user_roles ur
        JOIN rbac.roles r ON ur.role_id = r.role_id
        WHERE ur.user_id = $1 AND ur.is_active = TRUE AND r.is_active = TRUE
        AND (ur.expires_at IS NULL OR ur.expires_at > now())
        ORDER BY r.hierarchy_level DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let mut roles = Vec::new();
    let mut highest_role = UserRole::Guest;

    for role_data in roles_data {
        if let Some(role) = UserRole::from_slug(&role_data.role_slug) {
            if role.hierarchy_level() > highest_role.hierarchy_level() {
                highest_role = role.clone();
            }
            roles.push(role);
        }
    }

    // Get user's effective permissions
    let permissions_data = sqlx::query!(
        r#"
        SELECT permission_slug, sensitivity_level
        FROM rbac.user_effective_permissions
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let mut permissions = HashSet::new();
    for perm in permissions_data {
        permissions.insert(perm.permission_slug);
    }

    // Determine special access flags
    let can_access_secrets = permissions.contains("secrets.read") || 
                            permissions.contains("secrets.create") ||
                            permissions.contains("secrets.update") ||
                            permissions.contains("secrets.delete");

    let can_manage_users = permissions.contains("users.create") ||
                          permissions.contains("users.update") ||
                          permissions.contains("users.delete") ||
                          permissions.contains("users.roles");

    let can_admin_system = permissions.contains("system.admin") ||
                          permissions.contains("system.config") ||
                          permissions.contains("system.maintenance");

    debug!(
        "User {} permissions: {} roles, {} permissions, highest_role: {:?}, secrets: {}, users: {}, system: {}",
        user_id, roles.len(), permissions.len(), highest_role, can_access_secrets, can_manage_users, can_admin_system
    );

    Ok(UserPermissions {
        user_id,
        roles,
        permissions,
        highest_role,
        can_access_secrets,
        can_manage_users,
        can_admin_system,
    })
}

/// Check if user has specific permission
pub async fn user_has_permission(pool: &PgPool, user_id: Uuid, permission: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT rbac.user_has_permission($1, $2) as has_permission",
        user_id,
        permission
    )
    .fetch_one(pool)
    .await?;

    Ok(result.has_permission.unwrap_or(false))
}

/// Middleware to require specific permission
pub async fn require_permission_middleware<B>(
    req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
    permission: String,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    // Add permission requirement to request extensions
    let (mut parts, body) = req.into_parts();
    parts.extensions.insert(permission.clone());
    let req = axum::http::Request::from_parts(parts, body);

    // Continue with request
    Ok(next.run(req).await)
}

/// Middleware to require specific role
pub async fn require_role_middleware<B>(
    req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
    role: UserRole,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    // Add role requirement to request extensions
    let (mut parts, body) = req.into_parts();
    parts.extensions.insert(role);
    let req = axum::http::Request::from_parts(parts, body);

    // Continue with request
    Ok(next.run(req).await)
}

/// Helper macros for common permission checks
#[macro_export]
macro_rules! require_permission {
    ($permission:expr) => {
        axum::middleware::from_fn_with_state(
            $permission.to_string(),
            crate::middleware::rbac::require_permission_middleware
        )
    };
}

#[macro_export]
macro_rules! require_role {
    ($role:expr) => {
        axum::middleware::from_fn_with_state(
            $role,
            crate::middleware::rbac::require_role_middleware
        )
    };
}

/// Permission categories for easy reference
pub mod permissions {
    // System permissions
    pub const SYSTEM_ADMIN: &str = "system.admin";
    pub const SYSTEM_CONFIG: &str = "system.config";
    pub const SYSTEM_MAINTENANCE: &str = "system.maintenance";

    // User management
    pub const USERS_CREATE: &str = "users.create";
    pub const USERS_READ: &str = "users.read";
    pub const USERS_UPDATE: &str = "users.update";
    pub const USERS_DELETE: &str = "users.delete";
    pub const USERS_ROLES: &str = "users.roles";

    // Secrets and security
    pub const SECRETS_READ: &str = "secrets.read";
    pub const SECRETS_CREATE: &str = "secrets.create";
    pub const SECRETS_UPDATE: &str = "secrets.update";
    pub const SECRETS_DELETE: &str = "secrets.delete";

    // Agent management
    pub const AGENTS_READ: &str = "agents.read";
    pub const AGENTS_CREATE: &str = "agents.create";
    pub const AGENTS_UPDATE: &str = "agents.update";
    pub const AGENTS_DELETE: &str = "agents.delete";
    pub const AGENTS_CONTROL: &str = "agents.control";

    // Subscription management
    pub const SUBSCRIPTIONS_READ: &str = "subscriptions.read";
    pub const SUBSCRIPTIONS_CREATE: &str = "subscriptions.create";
    pub const SUBSCRIPTIONS_UPDATE: &str = "subscriptions.update";
    pub const SUBSCRIPTIONS_DELETE: &str = "subscriptions.delete";
    pub const SUBSCRIPTIONS_MIGRATE: &str = "subscriptions.migrate";

    // Security monitoring
    pub const SECURITY_INCIDENTS: &str = "security.incidents";
    pub const SECURITY_MONITORING: &str = "security.monitoring";
    pub const SECURITY_RESPONSE: &str = "security.response";

    // Audit and compliance
    pub const AUDIT_READ: &str = "audit.read";
    pub const AUDIT_EXPORT: &str = "audit.export";

    // API access
    pub const API_READ: &str = "api.read";
    pub const API_WRITE: &str = "api.write";
    pub const API_ADMIN: &str = "api.admin";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_hierarchy() {
        assert!(UserRole::SystemAdmin.hierarchy_level() > UserRole::Admin.hierarchy_level());
        assert!(UserRole::Admin.hierarchy_level() > UserRole::User.hierarchy_level());
        assert!(UserRole::User.hierarchy_level() > UserRole::Guest.hierarchy_level());
    }

    #[test]
    fn test_user_role_slug_conversion() {
        assert_eq!(UserRole::SystemAdmin.to_slug(), "system_admin");
        assert_eq!(UserRole::from_slug("system_admin"), Some(UserRole::SystemAdmin));
        assert_eq!(UserRole::from_slug("invalid"), None);
    }

    #[test]
    fn test_user_permissions_check() {
        let mut permissions = HashSet::new();
        permissions.insert("users.read".to_string());
        permissions.insert("agents.create".to_string());

        let user_perms = UserPermissions {
            user_id: Uuid::new_v4(),
            roles: vec![UserRole::User],
            permissions,
            highest_role: UserRole::User,
            can_access_secrets: false,
            can_manage_users: false,
            can_admin_system: false,
        };

        assert!(user_perms.has_permission("users.read"));
        assert!(!user_perms.has_permission("users.delete"));
        assert!(user_perms.has_any_permission(&["users.read", "users.delete"]));
        assert!(!user_perms.has_all_permissions(&["users.read", "users.delete"]));
    }
}