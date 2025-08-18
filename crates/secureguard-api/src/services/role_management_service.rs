use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use secureguard_shared::{SecureGuardError, Result};
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::middleware::rbac::{UserRole, UserPermissions};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub role_id: Uuid,
    pub role_name: String,
    pub role_slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub role_type: String,
    pub parent_role_id: Option<Uuid>,
    pub hierarchy_level: i32,
    pub is_active: bool,
    pub is_system_role: bool,
    pub is_assignable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub permission_id: Uuid,
    pub permission_name: String,
    pub permission_slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub resource_type: Option<String>,
    pub action: String,
    pub sensitivity_level: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleAssignment {
    pub user_role_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub context: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleInfo {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub primary_role: Option<String>,
    pub all_roles: Vec<String>,
    pub total_permissions: i64,
    pub high_sensitivity_permissions: i64,
    pub can_access_secrets: bool,
    pub can_manage_users: bool,
    pub can_admin_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRoleRequest {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRolePermissionsRequest {
    pub role_id: Uuid,
    pub add_permissions: Vec<Uuid>,
    pub remove_permissions: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermissionAudit {
    pub audit_id: Uuid,
    pub action: String, // assigned, removed, permission_added, permission_removed
    pub user_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub permission_id: Option<Uuid>,
    pub performed_by: Uuid,
    pub performed_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub ip_address: Option<String>,
}

pub struct RoleManagementService {
    pool: PgPool,
}

impl RoleManagementService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all available roles
    pub async fn get_all_roles(&self, include_inactive: bool) -> Result<Vec<Role>> {
        let roles = if include_inactive {
            sqlx::query_as!(
                Role,
                r#"
                SELECT role_id, role_name, role_slug, display_name, description,
                       role_type, parent_role_id, hierarchy_level, 
                       is_active, is_system_role, is_assignable,
                       created_at, updated_at
                FROM rbac.roles
                ORDER BY hierarchy_level DESC, role_name
                "#
            )
        } else {
            sqlx::query_as!(
                Role,
                r#"
                SELECT role_id, role_name, role_slug, display_name, description,
                       role_type, parent_role_id, hierarchy_level,
                       is_active, is_system_role, is_assignable,
                       created_at, updated_at
                FROM rbac.roles
                WHERE is_active = TRUE
                ORDER BY hierarchy_level DESC, role_name
                "#
            )
        }.fetch_all(&self.pool).await.map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(roles)
    }

    /// Get all permissions organized by category
    pub async fn get_all_permissions(&self) -> Result<HashMap<String, Vec<Permission>>> {
        let permissions = sqlx::query_as!(
            Permission,
            r#"
            SELECT permission_id, permission_name, permission_slug, display_name, 
                   description, category, resource_type, action, sensitivity_level, is_active
            FROM rbac.permissions
            WHERE is_active = TRUE
            ORDER BY category, sensitivity_level DESC, permission_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let mut categorized = HashMap::new();
        for permission in permissions {
            categorized
                .entry(permission.category.clone())
                .or_insert_with(Vec::new)
                .push(permission);
        }

        Ok(categorized)
    }

    /// Get user roles and permissions summary
    pub async fn get_user_roles_summary(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<UserRoleInfo>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let users = sqlx::query!(
            r#"
            SELECT 
                u.user_id, u.username, u.email,
                r.role_name as primary_role,
                array_agg(DISTINCT ar.role_name ORDER BY ar.hierarchy_level DESC) as all_roles,
                COUNT(DISTINCT p.permission_id) as total_permissions,
                COUNT(DISTINCT CASE WHEN p.sensitivity_level >= 3 THEN p.permission_id END) as high_sensitivity_permissions
            FROM users.users u
            LEFT JOIN rbac.roles r ON u.primary_role_id = r.role_id
            LEFT JOIN rbac.user_roles ur ON u.user_id = ur.user_id AND ur.is_active = TRUE
            LEFT JOIN rbac.roles ar ON ur.role_id = ar.role_id AND ar.is_active = TRUE
            LEFT JOIN rbac.user_effective_permissions p ON u.user_id = p.user_id
            GROUP BY u.user_id, u.username, u.email, r.role_name
            ORDER BY u.username
            LIMIT $1 OFFSET $2
            "#,
            limit, offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let mut user_infos = Vec::new();
        for user in users {
            // Check special permissions
            let special_perms = sqlx::query!(
                r#"
                SELECT 
                    bool_or(permission_slug IN ('secrets.read', 'secrets.create', 'secrets.update', 'secrets.delete')) as can_access_secrets,
                    bool_or(permission_slug IN ('users.create', 'users.update', 'users.delete', 'users.roles')) as can_manage_users,
                    bool_or(permission_slug IN ('system.admin', 'system.config', 'system.maintenance')) as can_admin_system
                FROM rbac.user_effective_permissions
                WHERE user_id = $1
                "#,
                user.user_id
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

            user_infos.push(UserRoleInfo {
                user_id: user.user_id,
                username: user.username,
                email: user.email,
                primary_role: user.primary_role,
                all_roles: user.all_roles.unwrap_or_default(),
                total_permissions: user.total_permissions.unwrap_or(0),
                high_sensitivity_permissions: user.high_sensitivity_permissions.unwrap_or(0),
                can_access_secrets: special_perms.can_access_secrets.unwrap_or(false),
                can_manage_users: special_perms.can_manage_users.unwrap_or(false),
                can_admin_system: special_perms.can_admin_system.unwrap_or(false),
            });
        }

        Ok(user_infos)
    }

    /// Get specific user's roles and permissions
    pub async fn get_user_role_details(&self, user_id: Uuid) -> Result<UserRoleInfo> {
        let user_summary = self.get_user_roles_summary(Some(1), Some(0)).await?;
        user_summary.into_iter()
            .find(|u| u.user_id == user_id)
            .ok_or_else(|| SecureGuardError::UserNotFound)
    }

    /// Assign role to user
    pub async fn assign_role_to_user(
        &self, 
        request: AssignRoleRequest, 
        assigned_by: Uuid,
        ip_address: Option<String>
    ) -> Result<Uuid> {
        info!("👤 Assigning role {} to user {}", request.role_id, request.user_id);

        // Check if role exists and is assignable
        let role = sqlx::query!(
            "SELECT role_name, is_assignable FROM rbac.roles WHERE role_id = $1 AND is_active = TRUE",
            request.role_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SecureGuardError::ValidationError("Role not found or inactive".to_string()))?;

        if !role.is_assignable {
            return Err(SecureGuardError::ValidationError("Role is not assignable".to_string()));
        }

        // Check if user already has this role
        let existing = sqlx::query!(
            "SELECT user_role_id FROM rbac.user_roles WHERE user_id = $1 AND role_id = $2 AND is_active = TRUE",
            request.user_id, request.role_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            return Err(SecureGuardError::ValidationError("User already has this role".to_string()));
        }

        // Assign the role
        let user_role_id = sqlx::query!(
            r#"
            INSERT INTO rbac.user_roles (user_id, role_id, assigned_by, expires_at, context)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING user_role_id
            "#,
            request.user_id,
            request.role_id,
            assigned_by,
            request.expires_at,
            request.context
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .user_role_id;

        // Log the assignment
        self.log_role_audit(
            "role_assigned",
            Some(request.user_id),
            Some(request.role_id),
            None,
            assigned_by,
            request.context.as_deref(),
            ip_address.as_deref(),
        ).await?;

        info!("✅ Role {} assigned to user {} successfully", role.role_name, request.user_id);
        Ok(user_role_id)
    }

    /// Remove role from user
    pub async fn remove_role_from_user(
        &self, 
        user_id: Uuid, 
        role_id: Uuid, 
        removed_by: Uuid,
        reason: Option<String>,
        ip_address: Option<String>
    ) -> Result<()> {
        info!("🗑️ Removing role {} from user {}", role_id, user_id);

        // Check if assignment exists
        let assignment = sqlx::query!(
            "SELECT user_role_id FROM rbac.user_roles WHERE user_id = $1 AND role_id = $2 AND is_active = TRUE",
            user_id, role_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SecureGuardError::ValidationError("Role assignment not found".to_string()))?;

        // Remove the role assignment
        sqlx::query!(
            "UPDATE rbac.user_roles SET is_active = FALSE WHERE user_role_id = $1",
            assignment.user_role_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Log the removal
        self.log_role_audit(
            "role_removed",
            Some(user_id),
            Some(role_id),
            None,
            removed_by,
            reason.as_deref(),
            ip_address.as_deref(),
        ).await?;

        info!("✅ Role {} removed from user {} successfully", role_id, user_id);
        Ok(())
    }

    /// Update role permissions
    pub async fn update_role_permissions(
        &self, 
        request: UpdateRolePermissionsRequest, 
        updated_by: Uuid,
        ip_address: Option<String>
    ) -> Result<()> {
        info!("🔧 Updating permissions for role {}", request.role_id);

        // Check if role exists and is not a system role
        let role = sqlx::query!(
            "SELECT role_name, is_system_role FROM rbac.roles WHERE role_id = $1 AND is_active = TRUE",
            request.role_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SecureGuardError::ValidationError("Role not found".to_string()))?;

        if role.is_system_role {
            return Err(SecureGuardError::ValidationError("Cannot modify system role permissions".to_string()));
        }

        // Add new permissions
        for permission_id in &request.add_permissions {
            // Check if permission already exists
            let exists = sqlx::query!(
                "SELECT 1 FROM rbac.role_permissions WHERE role_id = $1 AND permission_id = $2",
                request.role_id, permission_id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

            if exists.is_none() {
                sqlx::query!(
                    "INSERT INTO rbac.role_permissions (role_id, permission_id, granted_by) VALUES ($1, $2, $3)",
                    request.role_id, permission_id, updated_by
                )
                .execute(&self.pool)
                .await
                .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

                self.log_role_audit(
                    "permission_added",
                    None,
                    Some(request.role_id),
                    Some(*permission_id),
                    updated_by,
                    Some("Permission added to role"),
                    ip_address.as_deref(),
                ).await?;
            }
        }

        // Remove permissions
        for permission_id in &request.remove_permissions {
            sqlx::query!(
                "DELETE FROM rbac.role_permissions WHERE role_id = $1 AND permission_id = $2",
                request.role_id, permission_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

            self.log_role_audit(
                "permission_removed",
                None,
                Some(request.role_id),
                Some(*permission_id),
                updated_by,
                Some("Permission removed from role"),
                ip_address.as_deref(),
            ).await?;
        }

        info!("✅ Role {} permissions updated: +{} -{}", 
            role.role_name, request.add_permissions.len(), request.remove_permissions.len());

        Ok(())
    }

    /// Set user's primary role
    pub async fn set_user_primary_role(
        &self, 
        user_id: Uuid, 
        role_id: Uuid, 
        updated_by: Uuid
    ) -> Result<()> {
        info!("🎯 Setting primary role for user {} to {}", user_id, role_id);

        // Verify user has the role assigned
        let has_role = sqlx::query!(
            "SELECT 1 FROM rbac.user_roles WHERE user_id = $1 AND role_id = $2 AND is_active = TRUE",
            user_id, role_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if has_role.is_none() {
            return Err(SecureGuardError::ValidationError("User does not have this role assigned".to_string()));
        }

        // Update primary role
        sqlx::query!(
            "UPDATE users.users SET primary_role_id = $1 WHERE user_id = $2",
            role_id, user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        info!("✅ Primary role set for user {}", user_id);
        Ok(())
    }

    /// Get role audit trail
    pub async fn get_role_audit_trail(
        &self, 
        user_id: Option<Uuid>, 
        limit: Option<i64>
    ) -> Result<Vec<RolePermissionAudit>> {
        let limit = limit.unwrap_or(100);

        let audits = if let Some(uid) = user_id {
            sqlx::query!(
                r#"
                SELECT audit_id, action, user_id, role_id, permission_id,
                       performed_by, performed_at, reason, ip_address
                FROM rbac.role_audit_log
                WHERE user_id = $1
                ORDER BY performed_at DESC
                LIMIT $2
                "#,
                uid, limit
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query!(
                r#"
                SELECT audit_id, action, user_id, role_id, permission_id,
                       performed_by, performed_at, reason, ip_address
                FROM rbac.role_audit_log
                ORDER BY performed_at DESC
                LIMIT $1
                "#,
                limit
            )
            .fetch_all(&self.pool)
            .await
        }.map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let mut audit_records = Vec::new();
        for audit in audits {
            audit_records.push(RolePermissionAudit {
                audit_id: audit.audit_id,
                action: audit.action,
                user_id: audit.user_id,
                role_id: audit.role_id,
                permission_id: audit.permission_id,
                performed_by: audit.performed_by,
                performed_at: audit.performed_at,
                reason: audit.reason,
                ip_address: audit.ip_address,
            });
        }

        Ok(audit_records)
    }

    /// Check if user can access secrets
    pub async fn user_can_access_secrets(&self, user_id: Uuid) -> Result<bool> {
        let has_secret_access = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM rbac.user_effective_permissions
                WHERE user_id = $1 AND permission_slug IN ('secrets.read', 'secrets.create', 'secrets.update', 'secrets.delete')
            ) as can_access
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(has_secret_access.can_access.unwrap_or(false))
    }

    /// Get users with access to sensitive areas
    pub async fn get_users_with_sensitive_access(&self) -> Result<Vec<UserRoleInfo>> {
        let users = sqlx::query!(
            r#"
            SELECT DISTINCT u.user_id, u.username, u.email,
                   r.role_name as primary_role,
                   bool_or(uep.permission_slug IN ('secrets.read', 'secrets.create', 'secrets.update', 'secrets.delete')) as can_access_secrets,
                   bool_or(uep.permission_slug IN ('users.create', 'users.update', 'users.delete', 'users.roles')) as can_manage_users,
                   bool_or(uep.permission_slug IN ('system.admin', 'system.config', 'system.maintenance')) as can_admin_system
            FROM users.users u
            LEFT JOIN rbac.roles r ON u.primary_role_id = r.role_id
            JOIN rbac.user_effective_permissions uep ON u.user_id = uep.user_id
            WHERE uep.sensitivity_level >= 3
            GROUP BY u.user_id, u.username, u.email, r.role_name
            HAVING bool_or(uep.permission_slug IN ('secrets.read', 'secrets.create', 'secrets.update', 'secrets.delete', 'system.admin', 'users.roles'))
            ORDER BY u.username
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let mut sensitive_users = Vec::new();
        for user in users {
            sensitive_users.push(UserRoleInfo {
                user_id: user.user_id,
                username: user.username,
                email: user.email,
                primary_role: user.primary_role,
                all_roles: vec![], // Would need additional query
                total_permissions: 0, // Would need additional query
                high_sensitivity_permissions: 0, // Would need additional query
                can_access_secrets: user.can_access_secrets.unwrap_or(false),
                can_manage_users: user.can_manage_users.unwrap_or(false),
                can_admin_system: user.can_admin_system.unwrap_or(false),
            });
        }

        Ok(sensitive_users)
    }

    // Helper method to log role audit events
    async fn log_role_audit(
        &self,
        action: &str,
        user_id: Option<Uuid>,
        role_id: Option<Uuid>,
        permission_id: Option<Uuid>,
        performed_by: Uuid,
        reason: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO rbac.role_audit_log (action, user_id, role_id, permission_id, performed_by, reason, ip_address)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            action, user_id, role_id, permission_id, performed_by, reason, ip_address
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

// Add audit log table creation to migration
const AUDIT_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS rbac.role_audit_log (
    audit_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action VARCHAR(50) NOT NULL,
    user_id UUID REFERENCES users.users(user_id),
    role_id UUID REFERENCES rbac.roles(role_id),
    permission_id UUID REFERENCES rbac.permissions(permission_id),
    performed_by UUID NOT NULL REFERENCES users.users(user_id),
    performed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reason TEXT,
    ip_address INET,
    
    INDEX (user_id, performed_at),
    INDEX (performed_by, performed_at),
    INDEX (action, performed_at)
);
"#;